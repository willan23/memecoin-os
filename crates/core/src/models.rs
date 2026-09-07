use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenStatus {
    Listed,
    Watchlist,
    Discovered,
    Unverified,
    UnderReview,
    HighRisk,
    Rejected,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    Unverified,
    DataVerified,
    ContractVerified,
    EcosystemVerified,
    CommunityVerified,
}

impl VerificationLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationLevel::Unverified => "unverified",
            VerificationLevel::DataVerified => "data_verified",
            VerificationLevel::ContractVerified => "contract_verified",
            VerificationLevel::EcosystemVerified => "ecosystem_verified",
            VerificationLevel::CommunityVerified => "community_verified",
        }
    }
}

impl TokenStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenStatus::Listed => "listed",
            TokenStatus::Watchlist => "watchlist",
            TokenStatus::Discovered => "discovered",
            TokenStatus::Unverified => "unverified",
            TokenStatus::UnderReview => "under_review",
            TokenStatus::HighRisk => "high_risk",
            TokenStatus::Rejected => "rejected",
            TokenStatus::Disabled => "disabled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Valid,
    Stale,
    Missing,
    Duplicate,
    Conflict,
    Anomaly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DataState {
    Live,
    Recent,
    Stale,
    Conflict,
    Simulated,
    #[default]
    Missing,
}

impl DataState {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataState::Live => "live",
            DataState::Recent => "recent",
            DataState::Stale => "stale",
            DataState::Conflict => "conflict",
            DataState::Simulated => "simulated",
            DataState::Missing => "missing",
        }
    }

    /// True when a numeric observation exists (including stale/conflict/simulated).
    pub fn present(&self) -> bool {
        !matches!(self, DataState::Missing)
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "live" => DataState::Live,
            "recent" => DataState::Recent,
            "stale" => DataState::Stale,
            "conflict" => DataState::Conflict,
            "simulated" => DataState::Simulated,
            _ => DataState::Missing,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Moderate,
    High,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub provider: String,
    pub timestamp: DateTime<Utc>,
    pub freshness_secs: i64,
    pub confidence: f64,
    pub validation_status: ValidationStatus,
    pub raw_reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenContract {
    pub address: String,
    pub decimals: u8,
    pub explorer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainBinding {
    pub id: String,
    pub standard: String,
    #[serde(default)]
    pub is_primary: bool,
    pub contracts: ChainContracts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainContracts {
    pub token: TokenContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SocialLinks {
    pub twitter: Option<String>,
    pub telegram: Option<String>,
    pub github: Option<String>,
    pub discord: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSource {
    pub provider: String,
    pub provider_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSources {
    #[serde(default)]
    pub onchain: bool,
    #[serde(default)]
    pub social: bool,
    #[serde(default)]
    pub github: bool,
    #[serde(default)]
    pub exchanges: bool,
    #[serde(default)]
    pub news: bool,
    pub market: Option<MarketSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    #[serde(default = "default_true")]
    pub sentiment: bool,
    #[serde(default = "default_true")]
    pub whale_tracking: bool,
    #[serde(default = "default_true")]
    pub holder_analysis: bool,
    #[serde(default = "default_true")]
    pub growth_engine: bool,
    #[serde(default = "default_true")]
    pub ai_agent: bool,
    #[serde(default)]
    pub governance: bool,
    #[serde(default)]
    pub treasury: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            sentiment: true,
            whale_tracking: true,
            holder_analysis: true,
            growth_engine: true,
            ai_agent: true,
            governance: false,
            treasury: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Supply {
    pub circulating: Option<f64>,
    pub total: Option<f64>,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDefinition {
    pub schema_version: u32,
    pub token_id: String,
    pub symbol: String,
    pub name: String,
    #[serde(default = "default_listed")]
    pub status: TokenStatus,
    #[serde(default = "default_unverified")]
    pub verification: VerificationLevel,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub narratives: Vec<String>,
    pub launch_date: Option<String>,
    pub website: Option<String>,
    pub description: Option<String>,
    pub chains: Vec<ChainBinding>,
    #[serde(default)]
    pub socials: SocialLinks,
    pub data_sources: DataSources,
    #[serde(default)]
    pub features: FeatureFlags,
    #[serde(default)]
    pub supply: Supply,
}

fn default_listed() -> TokenStatus {
    TokenStatus::Listed
}

fn default_unverified() -> VerificationLevel {
    VerificationLevel::Unverified
}

impl TokenDefinition {
    pub fn primary_chain(&self) -> Option<&ChainBinding> {
        self.chains
            .iter()
            .find(|c| c.is_primary)
            .or_else(|| self.chains.first())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub price_usd: f64,
    pub market_cap_usd: f64,
    pub fdv_usd: f64,
    pub volume_24h_usd: f64,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub change_24h_pct: f64,
    pub ath_usd: Option<f64>,
    pub ath_distance_pct: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub provenance: Provenance,
    #[serde(default)]
    pub data_state: DataState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnchainSnapshot {
    pub holders: u64,
    pub active_holders_30d: u64,
    pub new_holders_7d: i64,
    pub retained_holders_30d_pct: f64,
    pub lost_holders_7d: u64,
    pub whale_holders: u64,
    pub top10_concentration_pct: f64,
    pub top50_concentration_pct: f64,
    pub transfers_24h: u64,
    pub unique_senders_24h: u64,
    pub large_transfers_24h: u64,
    pub exchange_inflow_usd: f64,
    pub exchange_outflow_usd: f64,
    pub provenance: Provenance,
    #[serde(default)]
    pub data_state: DataState,
    #[serde(default)]
    pub top_holders: Vec<HolderShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquiditySnapshot {
    pub liquidity_usd: f64,
    pub pool_count: u32,
    pub spread_bps: f64,
    pub depth_plus_2pct_usd: f64,
    pub depth_minus_2pct_usd: f64,
    pub lp_change_7d_pct: f64,
    pub buy_sell_imbalance: f64,
    pub provenance: Provenance,
    #[serde(default)]
    pub data_state: DataState,
    #[serde(default)]
    pub pools: Vec<DiscoveredPool>,
    #[serde(default)]
    pub counterparts: Vec<DiscoveryCandidate>,
    /// Official https links observed on DexScreener token info. Not a firehose.
    #[serde(default)]
    pub official_links: Vec<OfficialLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfficialLink {
    pub kind: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryCandidate {
    pub chain_id: String,
    pub address: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub pair_address: Option<String>,
    pub dex: Option<String>,
    pub liquidity_usd: Option<f64>,
    pub via_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HolderShare {
    pub address: String,
    pub share_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveredPool {
    pub chain_id: String,
    pub pair_address: String,
    pub dex: Option<String>,
    pub liquidity_usd: Option<f64>,
    pub price_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialSnapshot {
    pub mentions_24h: u64,
    pub unique_accounts_24h: u64,
    pub engagement_score: f64,
    pub sentiment_net: f64,
    pub bot_probability: f64,
    pub organicness: f64,
    pub narrative_diversity: f64,
    pub dominant_topics: Vec<String>,
    pub spam_ratio: f64,
    pub provenance: Provenance,
    #[serde(default)]
    pub data_state: DataState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentSnapshot {
    pub commits_30d: u64,
    pub active_contributors_30d: u64,
    pub releases_90d: u64,
    pub last_commit_days: Option<u32>,
    pub open_issues: u64,
    pub activity_score: f64,
    pub provenance: Provenance,
    #[serde(default)]
    pub data_state: DataState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreComponent {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub weight: f64,
    pub drivers: Vec<ScoreDriver>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDriver {
    pub label: String,
    pub delta: f64,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemScore {
    pub algorithm: String,
    pub algorithm_version: String,
    pub value: f64,
    pub confidence: f64,
    pub as_of: DateTime<Utc>,
    pub components: Vec<ScoreComponent>,
    pub why: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genome {
    pub algorithm_version: String,
    pub dimensions: Vec<GenomeDimension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeDimension {
    pub id: String,
    pub label: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub id: String,
    pub label: String,
    pub score: f64,
    pub level: RiskLevel,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskReport {
    pub algorithm: String,
    pub algorithm_version: String,
    pub score: f64,
    pub level: RiskLevel,
    pub confidence: f64,
    pub as_of: DateTime<Utc>,
    pub factors: Vec<RiskFactor>,
    pub why: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: String,
    pub title: String,
    pub impact: f64,
    pub cost: String,
    pub risk: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub expected_metric: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub at: DateTime<Utc>,
    pub kind: String,
    pub title: String,
    pub delta: Option<String>,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub token_id: String,
    pub kind: String,
    pub severity: String,
    pub title: String,
    pub body: String,
    pub evidence: Vec<String>,
    pub fired_at: DateTime<Utc>,
    #[serde(default)]
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub kind: String,
    pub url: String,
    pub chat_id: Option<String>,
    pub token_id: Option<String>,
    pub enabled: bool,
    pub digest_daily: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationCard {
    pub observation: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
    pub interpretation: String,
    pub risk: Option<String>,
    #[serde(default)]
    pub data_state: DataState,
    #[serde(default)]
    pub model_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyBriefing {
    pub token_id: String,
    pub as_of: DateTime<Utc>,
    pub ecosystem_health: f64,
    pub headline: String,
    pub changes: Vec<String>,
    pub risks: Vec<String>,
    pub opportunities: Vec<String>,
    pub investigation: String,
    pub confidence: f64,
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SparkPoint {
    pub t: String,
    pub v: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSnapshot {
    pub token: TokenSummary,
    pub market: MarketSnapshot,
    pub liquidity: LiquiditySnapshot,
    pub onchain: OnchainSnapshot,
    pub social: SocialSnapshot,
    pub development: DevelopmentSnapshot,
    pub scores: EcosystemScore,
    pub genome: Genome,
    pub risk: RiskReport,
    pub growth: Vec<Opportunity>,
    pub timeline: Vec<TimelineEvent>,
    pub briefing: DailyBriefing,
    pub observations: Vec<ObservationCard>,
    pub price_series: Vec<SparkPoint>,
    pub health_series: Vec<SparkPoint>,
    pub as_of: DateTime<Utc>,
    #[serde(default)]
    pub data_state: DataState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub status: TokenStatus,
    pub verification: VerificationLevel,
    pub narratives: Vec<String>,
    pub primary_chain: String,
    pub website: Option<String>,
    pub description: Option<String>,
    pub contract: Option<String>,
    pub chains: Vec<String>,
}
