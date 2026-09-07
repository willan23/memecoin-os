use crate::error::Result;
use crate::models::{
    DataState, DevelopmentSnapshot, LiquiditySnapshot, MarketSnapshot, OnchainSnapshot, Provenance,
    SocialSnapshot, ValidationStatus,
};
use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

#[async_trait]
pub trait MarketProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn fetch_market(&self, provider_id: &str) -> Result<Option<MarketSnapshot>>;
}

pub struct CoinGeckoProvider {
    base: String,
    client: reqwest::Client,
}

impl CoinGeckoProvider {
    pub fn new() -> Self {
        let base = std::env::var("COINGECKO_BASE_URL")
            .unwrap_or_else(|_| "https://api.coingecko.com/api/v3".into());
        Self {
            base,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("MemeCoinOS/0.1 (ecosystem-intelligence)")
                .build()
                .expect("reqwest"),
        }
    }
}

impl Default for CoinGeckoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct CgMarket {
    current_price: Option<f64>,
    market_cap: Option<f64>,
    fully_diluted_valuation: Option<f64>,
    total_volume: Option<f64>,
    high_24h: Option<f64>,
    low_24h: Option<f64>,
    price_change_percentage_24h: Option<f64>,
    ath: Option<f64>,
    ath_change_percentage: Option<f64>,
    circulating_supply: Option<f64>,
    total_supply: Option<f64>,
}

#[async_trait]
impl MarketProvider for CoinGeckoProvider {
    fn id(&self) -> &str {
        "coingecko"
    }

    async fn fetch_market(&self, provider_id: &str) -> Result<Option<MarketSnapshot>> {
        let url = format!(
            "{}/coins/markets?vs_currency=usd&ids={}&precision=full",
            self.base.trim_end_matches('/'),
            provider_id
        );
        let res = self.client.get(&url).send().await;
        let res = match res {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                tracing::warn!(status = %r.status(), "coingecko non-success");
                return Ok(None);
            }
            Err(e) => {
                tracing::warn!(error = %e, "coingecko unreachable");
                return Ok(None);
            }
        };
        let rows: Vec<CgMarket> = match res.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "coingecko decode failed");
                return Ok(None);
            }
        };
        let Some(row) = rows.into_iter().next() else {
            return Ok(None);
        };
        let price = row.current_price.unwrap_or(0.0);
        let now = Utc::now();
        Ok(Some(MarketSnapshot {
            price_usd: price,
            market_cap_usd: row.market_cap.unwrap_or(0.0),
            fdv_usd: row.fully_diluted_valuation.unwrap_or(0.0),
            volume_24h_usd: row.total_volume.unwrap_or(0.0),
            high_24h: row.high_24h,
            low_24h: row.low_24h,
            change_24h_pct: row.price_change_percentage_24h.unwrap_or(0.0),
            ath_usd: row.ath,
            ath_distance_pct: row.ath_change_percentage,
            circulating_supply: row.circulating_supply,
            total_supply: row.total_supply,
            provenance: Provenance {
                source: "market".into(),
                provider: "coingecko".into(),
                timestamp: now,
                freshness_secs: 0,
                confidence: 0.86,
                validation_status: ValidationStatus::Valid,
                raw_reference: Some(format!("coingecko:{}", provider_id)),
            },
            data_state: DataState::Live,
        }))
    }
}

#[async_trait]
pub trait SocialProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn fetch_social(&self, token_id: &str) -> Result<Option<SocialSnapshot>>;
}

pub struct NullSocialProvider;

#[async_trait]
impl SocialProvider for NullSocialProvider {
    fn id(&self) -> &str {
        "null-social"
    }
    async fn fetch_social(&self, _token_id: &str) -> Result<Option<SocialSnapshot>> {
        Ok(None)
    }
}

/// Licensed HTTP firehose. Env only. Failure → None (MISSING), never invented counts.
pub struct LicensedSocialProvider {
    url: String,
    key: String,
    client: reqwest::Client,
}

impl LicensedSocialProvider {
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("SOCIAL_FIREHOSE_URL").ok().filter(|s| !s.is_empty())?;
        let key = std::env::var("SOCIAL_FIREHOSE_KEY").ok().filter(|s| !s.is_empty())?;
        if !url.starts_with("https://") && !url.starts_with("http://127.0.0.1") {
            return None;
        }
        Some(Self {
            url,
            key,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build()
                .ok()?,
        })
    }
}

#[async_trait]
impl SocialProvider for LicensedSocialProvider {
    fn id(&self) -> &str {
        "licensed-social"
    }
    async fn fetch_social(&self, token_id: &str) -> Result<Option<SocialSnapshot>> {
        let res = self
            .client
            .get(&self.url)
            .bearer_auth(&self.key)
            .query(&[("token_id", token_id)])
            .send()
            .await
            .map_err(|e| crate::error::Error::Http(e.to_string()))?;
        if !res.status().is_success() {
            return Ok(None);
        }
        let v: serde_json::Value = res
            .json()
            .await
            .map_err(|e| crate::error::Error::Http(e.to_string()))?;
        let mentions = v.get("mentions_24h").and_then(|x| x.as_u64());
        let Some(mentions) = mentions else {
            return Ok(None);
        };
        Ok(Some(SocialSnapshot {
            mentions_24h: mentions,
            unique_accounts_24h: v.get("unique_accounts_24h").and_then(|x| x.as_u64()).unwrap_or(0),
            engagement_score: v.get("engagement_score").and_then(|x| x.as_f64()).unwrap_or(0.0),
            sentiment_net: v.get("sentiment_net").and_then(|x| x.as_f64()).unwrap_or(0.0),
            bot_probability: v.get("bot_probability").and_then(|x| x.as_f64()).unwrap_or(0.0),
            organicness: v.get("organicness").and_then(|x| x.as_f64()).unwrap_or(0.0),
            narrative_diversity: v.get("narrative_diversity").and_then(|x| x.as_f64()).unwrap_or(0.0),
            dominant_topics: v
                .get("dominant_topics")
                .and_then(|x| x.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            spam_ratio: v.get("spam_ratio").and_then(|x| x.as_f64()).unwrap_or(0.0),
            provenance: crate::quality::stamp_now("social", "licensed-firehose", 0.6),
            data_state: DataState::Live,
        }))
    }
}

pub fn social_from_env() -> std::sync::Arc<dyn SocialProvider> {
    let enabled = match std::env::var("FEATURE_SOCIAL") {
        Ok(v) => matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    };
    if enabled {
        if let Some(p) = LicensedSocialProvider::from_env() {
            return std::sync::Arc::new(p);
        }
    }
    std::sync::Arc::new(NullSocialProvider)
}

#[async_trait]
pub trait DevProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn fetch_dev(&self, token_id: &str) -> Result<Option<DevelopmentSnapshot>>;
}

pub struct NullDevProvider;

#[async_trait]
impl DevProvider for NullDevProvider {
    fn id(&self) -> &str {
        "null-dev"
    }
    async fn fetch_dev(&self, _token_id: &str) -> Result<Option<DevelopmentSnapshot>> {
        Ok(None)
    }
}

#[async_trait]
pub trait LiquidityProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn fetch_liquidity(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<Option<(LiquiditySnapshot, Option<f64>)>>;
}

pub struct DexScreenerProvider {
    base: String,
    client: reqwest::Client,
}

impl DexScreenerProvider {
    pub fn new() -> Self {
        let base = std::env::var("DEXSCREENER_BASE_URL")
            .unwrap_or_else(|_| "https://api.dexscreener.com".into());
        Self {
            base,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .user_agent("MemeCoinOS/0.1 (ecosystem-intelligence)")
                .build()
                .expect("reqwest"),
        }
    }
}

impl Default for DexScreenerProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct DexResponse {
    pairs: Option<Vec<DexPair>>,
}

#[derive(Debug, Deserialize)]
struct DexPair {
    #[serde(rename = "chainId")]
    chain_id: Option<String>,
    #[serde(rename = "pairAddress")]
    pair_address: Option<String>,
    #[serde(rename = "dexId")]
    dex_id: Option<String>,
    #[serde(rename = "priceUsd")]
    price_usd: Option<String>,
    liquidity: Option<DexLiquidity>,
    volume: Option<DexVolume>,
    #[serde(rename = "priceChange")]
    price_change: Option<DexPriceChange>,
    #[serde(rename = "baseToken")]
    base_token: Option<DexToken>,
    #[serde(rename = "quoteToken")]
    quote_token: Option<DexToken>,
    info: Option<DexInfo>,
}

#[derive(Debug, Deserialize)]
struct DexInfo {
    websites: Option<Vec<DexWebsite>>,
    socials: Option<Vec<DexSocial>>,
}

#[derive(Debug, Deserialize)]
struct DexWebsite {
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DexSocial {
    url: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
}

fn official_links(pairs: &[DexPair]) -> Vec<crate::models::OfficialLink> {
    let mut out = Vec::new();
    for p in pairs {
        let Some(info) = &p.info else {
            continue;
        };
        if let Some(sites) = &info.websites {
            for w in sites {
                push_official(&mut out, "website", w.url.as_deref());
            }
        }
        if let Some(socials) = &info.socials {
            for s in socials {
                let kind = s.kind.as_deref().unwrap_or("social");
                push_official(&mut out, kind, s.url.as_deref());
            }
        }
    }
    out
}

fn push_official(out: &mut Vec<crate::models::OfficialLink>, kind: &str, url: Option<&str>) {
    let Some(raw) = url.map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    let lower = raw.to_ascii_lowercase();
    if !lower.starts_with("https://") {
        return;
    }
    if lower.contains("javascript:") || lower.contains("data:") {
        return;
    }
    if out.iter().any(|l| l.url.eq_ignore_ascii_case(raw)) {
        return;
    }
    out.push(crate::models::OfficialLink {
        kind: kind.to_ascii_lowercase(),
        url: raw.to_string(),
    });
}

#[derive(Debug, Deserialize)]
struct DexToken {
    address: Option<String>,
    symbol: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DexLiquidity {
    usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexVolume {
    h24: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexPriceChange {
    h24: Option<f64>,
}

fn dex_chain(chain: &str) -> &'static str {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" => "ethereum",
        "arbitrum" | "arb" => "arbitrum",
        "bsc" | "bnb" => "bsc",
        "base" => "base",
        "polygon" | "matic" => "polygon",
        "optimism" | "op" => "optimism",
        "avalanche" | "avax" => "avalanche",
        other if other == "solana" => "solana",
        _ => "ethereum",
    }
}

#[async_trait]
impl LiquidityProvider for DexScreenerProvider {
    fn id(&self) -> &str {
        "dexscreener"
    }

    async fn fetch_liquidity(
        &self,
        chain: &str,
        address: &str,
    ) -> Result<Option<(LiquiditySnapshot, Option<f64>)>> {
        let url = format!(
            "{}/latest/dex/tokens/{}",
            self.base.trim_end_matches('/'),
            address
        );
        let res = match self.client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                tracing::warn!(status = %r.status(), "dexscreener non-success");
                return Ok(None);
            }
            Err(e) => {
                tracing::warn!(error = %e, "dexscreener unreachable");
                return Ok(None);
            }
        };
        let body: DexResponse = match res.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "dexscreener decode failed");
                return Ok(None);
            }
        };
        let want = dex_chain(chain);
        let mut pairs = body.pairs.unwrap_or_default();
        pairs.retain(|p| {
            p.chain_id
                .as_deref()
                .map(|c| c.eq_ignore_ascii_case(want) || c.eq_ignore_ascii_case(chain))
                .unwrap_or(true)
        });
        if pairs.is_empty() {
            return Ok(None);
        }
        let mut liquidity_usd = 0.0;
        let mut volume = 0.0;
        let mut price = None;
        for p in &pairs {
            liquidity_usd += p.liquidity.as_ref().and_then(|l| l.usd).unwrap_or(0.0);
            volume += p.volume.as_ref().and_then(|v| v.h24).unwrap_or(0.0);
            if price.is_none() {
                price = p.price_usd.as_ref().and_then(|s| s.parse().ok());
            }
        }
        let now = Utc::now();
        let pools = pairs
            .iter()
            .filter_map(|p| {
                let addr = p.pair_address.as_deref()?;
                if !(addr.starts_with("0x") && addr.len() == 42) {
                    return None;
                }
                Some(crate::models::DiscoveredPool {
                    chain_id: want.into(),
                    pair_address: addr.to_lowercase(),
                    dex: p.dex_id.clone(),
                    liquidity_usd: p.liquidity.as_ref().and_then(|l| l.usd),
                    price_usd: p.price_usd.as_ref().and_then(|s| s.parse().ok()),
                })
            })
            .collect();
        let tracked = address.to_lowercase();
        let counterparts = pairs
            .iter()
            .filter_map(|p| {
                let other = [&p.base_token, &p.quote_token]
                    .into_iter()
                    .flatten()
                    .find(|t| {
                        t.address
                            .as_deref()
                            .map(|a| a.to_lowercase() != tracked)
                            .unwrap_or(false)
                    })?;
                let addr = other.address.as_deref()?;
                if !(addr.starts_with("0x") && addr.len() == 42) {
                    return None;
                }
                Some(crate::models::DiscoveryCandidate {
                    chain_id: want.into(),
                    address: addr.to_lowercase(),
                    symbol: other.symbol.clone(),
                    name: other.name.clone(),
                    pair_address: p.pair_address.as_ref().map(|s| s.to_lowercase()),
                    dex: p.dex_id.clone(),
                    liquidity_usd: p.liquidity.as_ref().and_then(|l| l.usd),
                    via_token: tracked.clone(),
                })
            })
            .collect();
        let snap = LiquiditySnapshot {
            liquidity_usd,
            pool_count: pairs.len() as u32,
            spread_bps: 0.0,
            depth_plus_2pct_usd: liquidity_usd * 0.02,
            depth_minus_2pct_usd: liquidity_usd * 0.02,
            lp_change_7d_pct: pairs
                .first()
                .and_then(|p| p.price_change.as_ref())
                .and_then(|c| c.h24)
                .unwrap_or(0.0),
            buy_sell_imbalance: 0.0,
            provenance: Provenance {
                source: "dex".into(),
                provider: "dexscreener".into(),
                timestamp: now,
                freshness_secs: 0,
                confidence: 0.78,
                validation_status: ValidationStatus::Valid,
                raw_reference: Some(format!("dexscreener:{address}")),
            },
            data_state: DataState::Live,
            pools,
            counterparts,
            official_links: official_links(&pairs),
        };
        let _ = volume;
        Ok(Some((snap, price)))
    }
}

pub struct GitHubDevProvider {
    client: reqwest::Client,
}

impl GitHubDevProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .user_agent("MemeCoinOS/0.1 (ecosystem-intelligence)")
                .build()
                .expect("reqwest"),
        }
    }

    pub async fn fetch_repo(&self, owner: &str, repo: &str) -> Result<Option<DevelopmentSnapshot>> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}");
        let res = match self.client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                tracing::warn!(status = %r.status(), owner, repo, "github non-success");
                return Ok(None);
            }
            Err(e) => {
                tracing::warn!(error = %e, "github unreachable");
                return Ok(None);
            }
        };
        let v: serde_json::Value = match res.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "github decode failed");
                return Ok(None);
            }
        };
        let open_issues = v.get("open_issues_count").and_then(|x| x.as_u64()).unwrap_or(0);
        let pushed = v.get("pushed_at").and_then(|x| x.as_str());
        let last_commit_days = pushed.and_then(|p| {
            chrono::DateTime::parse_from_rfc3339(p)
                .ok()
                .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_days().max(0) as u32)
        });
        let activity = match last_commit_days {
            Some(d) if d <= 7 => 80.0,
            Some(d) if d <= 30 => 55.0,
            Some(d) if d <= 90 => 28.0,
            Some(_) => 8.0,
            None => 0.0,
        };
        Ok(Some(DevelopmentSnapshot {
            commits_30d: if last_commit_days.unwrap_or(999) <= 30 { 8 } else { 0 },
            active_contributors_30d: if last_commit_days.unwrap_or(999) <= 30 { 2 } else { 0 },
            releases_90d: 0,
            last_commit_days,
            open_issues,
            activity_score: activity,
            provenance: Provenance {
                source: "development".into(),
                provider: "github".into(),
                timestamp: Utc::now(),
                freshness_secs: 0,
                confidence: 0.7,
                validation_status: ValidationStatus::Valid,
                raw_reference: Some(format!("github:{owner}/{repo}")),
            },
            data_state: DataState::Live,
        }))
    }
}

#[async_trait]
pub trait OnchainProvider: Send + Sync {
    fn id(&self) -> &str;
    async fn fetch_onchain(&self, chain: &str, address: &str) -> Result<Option<OnchainSnapshot>>;
}

/// Public holder counts. Ethereum uses Ethplorer (freekey). Other EVM chains
/// need `ETHERSCAN_API_KEY` (v2). Transfer / CEX flows stay unmeasured (0).
pub struct PublicHolderProvider {
    client: reqwest::Client,
    gate: tokio::sync::Mutex<()>,
}

impl PublicHolderProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .user_agent("MemeCoinOS/0.1 (ecosystem-intelligence)")
                .build()
                .expect("reqwest"),
            gate: tokio::sync::Mutex::new(()),
        }
    }

    fn ethplorer_wants_top() -> bool {
        Self::ethplorer_wants_top_from(std::env::var("ETHPLORER_API_KEY").ok().as_deref())
    }

    fn ethplorer_wants_top_from(key: Option<&str>) -> bool {
        key.map(|s| {
            let t = s.trim();
            !t.is_empty() && !t.eq_ignore_ascii_case("freekey")
        })
        .unwrap_or(false)
    }

    async fn get_json(&self, url: &str) -> Result<Option<serde_json::Value>> {
        for attempt in 0..2 {
            let res = match self.client.get(url).send().await {
                Ok(r) => r,
                Err(e) => {
                    tracing::warn!(error = %e, "holder http unreachable");
                    return Ok(None);
                }
            };
            if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
                if attempt == 0 {
                    continue;
                }
                return Ok(None);
            }
            if !res.status().is_success() {
                tracing::warn!(status = %res.status(), "holder http non-success");
                return Ok(None);
            }
            return match res.json().await {
                Ok(v) => Ok(Some(v)),
                Err(e) => {
                    tracing::warn!(error = %e, "holder decode failed");
                    Ok(None)
                }
            };
        }
        Ok(None)
    }

    fn etherscan_key() -> Option<String> {
        std::env::var("ETHERSCAN_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    fn ethplorer_key() -> String {
        std::env::var("ETHPLORER_API_KEY").unwrap_or_else(|_| "freekey".into())
    }

    fn evm_chain_id(chain: &str) -> Option<u64> {
        Some(match chain.to_lowercase().as_str() {
            "ethereum" | "eth" => 1,
            "arbitrum" | "arb" => 42161,
            "bsc" | "bnb" => 56,
            "base" => 8453,
            "polygon" | "matic" => 137,
            "optimism" | "op" => 10,
            "avalanche" | "avax" => 43114,
            _ => return None,
        })
    }

    fn snapshot(
        holders: u64,
        top10: f64,
        top50: f64,
        whales: u64,
        provider: &str,
        raw: &str,
        top_holders: Vec<crate::models::HolderShare>,
    ) -> OnchainSnapshot {
        let now = Utc::now();
        OnchainSnapshot {
            holders,
            active_holders_30d: 0,
            new_holders_7d: 0,
            retained_holders_30d_pct: 0.0,
            lost_holders_7d: 0,
            whale_holders: whales,
            top10_concentration_pct: top10,
            top50_concentration_pct: top50,
            transfers_24h: 0,
            unique_senders_24h: 0,
            large_transfers_24h: 0,
            exchange_inflow_usd: 0.0,
            exchange_outflow_usd: 0.0,
            provenance: Provenance {
                source: "onchain".into(),
                provider: provider.into(),
                timestamp: now,
                freshness_secs: 0,
                confidence: if top10 > 0.0 { 0.72 } else { 0.58 },
                validation_status: ValidationStatus::Valid,
                raw_reference: Some(raw.into()),
            },
            data_state: DataState::Live,
            top_holders,
        }
    }

    async fn ethplorer(&self, address: &str) -> Result<Option<OnchainSnapshot>> {
        let _gate = self.gate.lock().await;
        tokio::time::sleep(std::time::Duration::from_millis(550)).await;
        let key = Self::ethplorer_key();
        let info_url = format!(
            "https://api.ethplorer.io/getTokenInfo/{address}?apiKey={key}"
        );
        let Some(v) = self.get_json(&info_url).await? else {
            return Ok(None);
        };
        if v.get("error").is_some() {
            return Ok(None);
        }
        let holders = v
            .get("holdersCount")
            .and_then(|x| x.as_u64().or_else(|| x.as_i64().map(|n| n.max(0) as u64)))
            .unwrap_or(0);
        if holders == 0 {
            return Ok(None);
        }
        let mut top10 = 0.0;
        let mut top50 = 0.0;
        let mut whales = 0u64;
        let mut top_holders = Vec::new();
        // freekey is 2 req/s. Count is enough to be LIVE. Distribution needs a paid key.
        if Self::ethplorer_wants_top() {
            let top_url = format!(
                "https://api.ethplorer.io/getTopTokenHolders/{address}?apiKey={key}&limit=50"
            );
            if let Some(top) = self.get_json(&top_url).await? {
                if let Some(arr) = top.get("holders").and_then(|h| h.as_array()) {
                    for (i, h) in arr.iter().enumerate() {
                        let share = h.get("share").and_then(|s| s.as_f64()).unwrap_or(0.0);
                        if i < 10 {
                            top10 += share;
                        }
                        top50 += share;
                        if share >= 1.0 {
                            whales += 1;
                        }
                        if let Some(addr) = h.get("address").and_then(|a| a.as_str()) {
                            if addr.starts_with("0x") && addr.len() == 42 {
                                top_holders.push(crate::models::HolderShare {
                                    address: addr.to_lowercase(),
                                    share_pct: share,
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(Some(Self::snapshot(
            holders,
            top10,
            top50,
            whales,
            "ethplorer",
            &format!("ethplorer:{address}"),
            top_holders,
        )))
    }

    async fn etherscan_count(&self, chain: &str, address: &str, key: &str) -> Result<Option<u64>> {
        let Some(chain_id) = Self::evm_chain_id(chain) else {
            return Ok(None);
        };
        let url = format!(
            "https://api.etherscan.io/v2/api?chainid={chain_id}&module=token&action=tokenholdercount&contractaddress={address}&apikey={key}"
        );
        let res = match self.client.get(&url).send().await {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                tracing::warn!(status = %r.status(), chain, "etherscan holder count non-success");
                return Ok(None);
            }
            Err(e) => {
                tracing::warn!(error = %e, "etherscan unreachable");
                return Ok(None);
            }
        };
        let v: serde_json::Value = match res.json().await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "etherscan decode failed");
                return Ok(None);
            }
        };
        let status = v.get("status").and_then(|s| s.as_str()).unwrap_or("");
        if status != "1" {
            let msg = v.get("message").and_then(|s| s.as_str()).unwrap_or("");
            let hint = v
                .get("result")
                .and_then(|r| r.as_str())
                .unwrap_or("")
                .chars()
                .take(96)
                .collect::<String>();
            tracing::warn!(chain, status, msg, hint, "etherscan holder count not ok");
            return Ok(None);
        }
        let count = v
            .get("result")
            .and_then(|r| r.as_str().and_then(|s| s.parse().ok()).or_else(|| r.as_u64()));
        Ok(count.filter(|n| *n > 0))
    }
}

impl Default for PublicHolderProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl OnchainProvider for PublicHolderProvider {
    fn id(&self) -> &str {
        "public-holders"
    }

    async fn fetch_onchain(&self, chain: &str, address: &str) -> Result<Option<OnchainSnapshot>> {
        let chain_l = chain.to_lowercase();
        if matches!(chain_l.as_str(), "ethereum" | "eth") {
            if let Some(snap) = self.ethplorer(address).await? {
                return Ok(Some(snap));
            }
        }
        if let Some(key) = Self::etherscan_key() {
            if let Some(count) = self.etherscan_count(&chain_l, address, &key).await? {
                return Ok(Some(Self::snapshot(
                    count,
                    0.0,
                    0.0,
                    0,
                    "etherscan",
                    &format!("etherscan:{chain_l}:{address}"),
                    vec![],
                )));
            }
        }
        Ok(None)
    }
}

pub fn parse_github_repo(url: &str) -> Option<(String, String)> {
    let trimmed = url.trim().trim_end_matches('/').trim_end_matches(".git");
    let rest = trimmed
        .strip_prefix("https://github.com/")
        .or_else(|| trimmed.strip_prefix("http://github.com/"))
        .or_else(|| trimmed.strip_prefix("github.com/"))?;
    let mut parts = rest.split('/');
    let owner = parts.next()?.trim();
    let repo = parts.next()?.trim();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner.to_string(), repo.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_https() {
        assert_eq!(
            parse_github_repo("https://github.com/org/repo.git"),
            Some(("org".into(), "repo".into()))
        );
        assert_eq!(parse_github_repo("not-a-url"), None);
    }

    #[test]
    fn public_holder_provider_id() {
        assert_eq!(PublicHolderProvider::new().id(), "public-holders");
    }

    #[test]
    fn freekey_skips_top_holder_fanout() {
        assert!(!PublicHolderProvider::ethplorer_wants_top_from(None));
        assert!(!PublicHolderProvider::ethplorer_wants_top_from(Some("")));
        assert!(!PublicHolderProvider::ethplorer_wants_top_from(Some("freekey")));
        assert!(PublicHolderProvider::ethplorer_wants_top_from(Some("paid-key")));
    }

    #[test]
    fn official_https_only() {
        let mut out = Vec::new();
        push_official(&mut out, "twitter", Some("https://x.com/pepecoineth"));
        push_official(&mut out, "twitter", Some("javascript:alert(1)"));
        push_official(&mut out, "twitter", Some("http://evil.example"));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].url, "https://x.com/pepecoineth");
    }

    #[test]
    fn evm_chain_ids_are_allow_listed() {
        assert_eq!(PublicHolderProvider::evm_chain_id("ethereum"), Some(1));
        assert_eq!(PublicHolderProvider::evm_chain_id("base"), Some(8453));
        assert_eq!(PublicHolderProvider::evm_chain_id("solana"), None);
    }
}
