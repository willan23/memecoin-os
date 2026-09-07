use crate::models::{
    DataState, DevelopmentSnapshot, LiquiditySnapshot, MarketSnapshot, OnchainSnapshot,
    SocialSnapshot, SparkPoint, TimelineEvent, TokenDefinition,
};
use crate::quality::stamp_now;
use chrono::{Duration, Utc};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Curated, versioned observations used when a live provider is unavailable
/// and as the on-chain/social/dev layer until indexers are connected.
/// Every fixture is tagged with provider `fixture-v1` so the UI never implies
/// false precision.
pub struct TokenFixtures {
    pub market: MarketSnapshot,
    pub liquidity: LiquiditySnapshot,
    pub onchain: OnchainSnapshot,
    pub social: SocialSnapshot,
    pub development: DevelopmentSnapshot,
}

pub fn for_token(def: &TokenDefinition) -> TokenFixtures {
    let mut fx = match def.token_id.as_str() {
        "arbdoge-ai" => aidoge(),
        "baby-doge" => babydoge(),
        "kishu-inu" => kishu(),
        _ => generic(def),
    };
    fx.market.data_state = DataState::Simulated;
    fx.liquidity.data_state = DataState::Simulated;
    fx.onchain.data_state = DataState::Simulated;
    fx.social.data_state = DataState::Simulated;
    fx.development.data_state = DataState::Simulated;
    fx
}

fn aidoge() -> TokenFixtures {
    TokenFixtures {
        market: MarketSnapshot {
            price_usd: 1.0801715968107662e-11,
            market_cap_usd: 2_018_580.0,
            fdv_usd: 2_018_580.0,
            volume_24h_usd: 88_031.0,
            high_24h: Some(1.2012e-11),
            low_24h: Some(1.0347e-11),
            change_24h_pct: -8.0674,
            ath_usd: Some(1.118e-9),
            ath_distance_pct: Some(-99.03),
            circulating_supply: Some(1.8698609809544144e17),
            total_supply: Some(1.8698609809544144e17),
            provenance: stamp_now("market", "fixture-v1", 0.62),
            data_state: DataState::Simulated,
        },
        liquidity: LiquiditySnapshot {
            liquidity_usd: 412_000.0,
            pool_count: 4,
            spread_bps: 86.0,
            depth_plus_2pct_usd: 38_400.0,
            depth_minus_2pct_usd: 41_200.0,
            lp_change_7d_pct: -6.4,
            buy_sell_imbalance: -0.18,
            provenance: stamp_now("dex", "fixture-v1", 0.58),
            data_state: DataState::Simulated,
            pools: vec![],
            counterparts: vec![],
            official_links: vec![],
        },
        onchain: OnchainSnapshot {
            holders: 273_851,
            active_holders_30d: 18_420,
            new_holders_7d: 410,
            retained_holders_30d_pct: 61.0,
            lost_holders_7d: 680,
            whale_holders: 42,
            top10_concentration_pct: 38.4,
            top50_concentration_pct: 61.2,
            transfers_24h: 1_940,
            unique_senders_24h: 1_120,
            large_transfers_24h: 6,
            exchange_inflow_usd: 41_200.0,
            exchange_outflow_usd: 18_600.0,
            provenance: stamp_now("onchain", "fixture-v1", 0.64),
            data_state: DataState::Simulated,
            top_holders: vec![],
        },
        social: SocialSnapshot {
            mentions_24h: 420,
            unique_accounts_24h: 190,
            engagement_score: 44.0,
            sentiment_net: -0.12,
            bot_probability: 0.29,
            organicness: 0.61,
            narrative_diversity: 0.48,
            dominant_topics: vec!["ai".into(), "arbitrum".into(), "fud".into()],
            spam_ratio: 0.16,
            provenance: stamp_now("social", "fixture-v1", 0.55),
            data_state: DataState::Simulated,
        },
        development: DevelopmentSnapshot {
            commits_30d: 4,
            active_contributors_30d: 2,
            releases_90d: 0,
            last_commit_days: Some(48),
            open_issues: 3,
            activity_score: 22.0,
            provenance: stamp_now("dev", "fixture-v1", 0.5),
            data_state: DataState::Simulated,
        },
    }
}

fn babydoge() -> TokenFixtures {
    TokenFixtures {
        market: MarketSnapshot {
            price_usd: 3.7134092740850367e-10,
            market_cap_usd: 68_181_843.0,
            fdv_usd: 155_981_693.0,
            volume_24h_usd: 2_434_818.0,
            high_24h: Some(3.88101e-10),
            low_24h: Some(3.66738e-10),
            change_24h_pct: -1.49737,
            ath_usd: Some(6.498e-9),
            ath_distance_pct: Some(-94.29),
            circulating_supply: Some(1.8358804594332454e17),
            total_supply: Some(4.2e17),
            provenance: stamp_now("market", "fixture-v1", 0.62),
            data_state: DataState::Simulated,
        },
        liquidity: LiquiditySnapshot {
            liquidity_usd: 8_640_000.0,
            pool_count: 18,
            spread_bps: 22.0,
            depth_plus_2pct_usd: 640_000.0,
            depth_minus_2pct_usd: 610_000.0,
            lp_change_7d_pct: 2.1,
            buy_sell_imbalance: -0.04,
            provenance: stamp_now("dex", "fixture-v1", 0.7),
            data_state: DataState::Simulated,
            pools: vec![],
            counterparts: vec![],
            official_links: vec![],
        },
        onchain: OnchainSnapshot {
            holders: 1_250_000,
            active_holders_30d: 86_400,
            new_holders_7d: 3_120,
            retained_holders_30d_pct: 74.0,
            lost_holders_7d: 1_880,
            whale_holders: 86,
            top10_concentration_pct: 24.8,
            top50_concentration_pct: 46.1,
            transfers_24h: 18_600,
            unique_senders_24h: 9_240,
            large_transfers_24h: 14,
            exchange_inflow_usd: 420_000.0,
            exchange_outflow_usd: 390_000.0,
            provenance: stamp_now("onchain", "fixture-v1", 0.68),
            data_state: DataState::Simulated,
            top_holders: vec![],
        },
        social: SocialSnapshot {
            mentions_24h: 3_840,
            unique_accounts_24h: 1_620,
            engagement_score: 71.0,
            sentiment_net: 0.18,
            bot_probability: 0.17,
            organicness: 0.74,
            narrative_diversity: 0.66,
            dominant_topics: vec!["community".into(), "charity".into(), "listing".into()],
            spam_ratio: 0.09,
            provenance: stamp_now("social", "fixture-v1", 0.63),
            data_state: DataState::Simulated,
        },
        development: DevelopmentSnapshot {
            commits_30d: 28,
            active_contributors_30d: 9,
            releases_90d: 3,
            last_commit_days: Some(4),
            open_issues: 17,
            activity_score: 64.0,
            provenance: stamp_now("dev", "fixture-v1", 0.66),
            data_state: DataState::Simulated,
        },
    }
}

fn kishu() -> TokenFixtures {
    TokenFixtures {
        market: MarketSnapshot {
            price_usd: 1.1106744643592503e-10,
            market_cap_usd: 10_712_990.0,
            fdv_usd: 10_712_988.0,
            volume_24h_usd: 95_185.0,
            high_24h: Some(1.14772e-10),
            low_24h: Some(1.08363e-10),
            change_24h_pct: -1.38967,
            ath_usd: Some(1.7547e-8),
            ath_distance_pct: Some(-99.37),
            circulating_supply: Some(9.646214638863418e16),
            total_supply: Some(9.646214638863418e16),
            provenance: stamp_now("market", "fixture-v1", 0.62),
            data_state: DataState::Simulated,
        },
        liquidity: LiquiditySnapshot {
            liquidity_usd: 1_180_000.0,
            pool_count: 6,
            spread_bps: 54.0,
            depth_plus_2pct_usd: 92_000.0,
            depth_minus_2pct_usd: 88_400.0,
            lp_change_7d_pct: -3.2,
            buy_sell_imbalance: -0.09,
            provenance: stamp_now("dex", "fixture-v1", 0.6),
            data_state: DataState::Simulated,
            pools: vec![],
            counterparts: vec![],
            official_links: vec![],
        },
        onchain: OnchainSnapshot {
            holders: 148_200,
            active_holders_30d: 9_640,
            new_holders_7d: 180,
            retained_holders_30d_pct: 58.0,
            lost_holders_7d: 420,
            whale_holders: 31,
            top10_concentration_pct: 41.6,
            top50_concentration_pct: 63.8,
            transfers_24h: 640,
            unique_senders_24h: 310,
            large_transfers_24h: 3,
            exchange_inflow_usd: 22_400.0,
            exchange_outflow_usd: 11_100.0,
            provenance: stamp_now("onchain", "fixture-v1", 0.6),
            data_state: DataState::Simulated,
            top_holders: vec![],
        },
        social: SocialSnapshot {
            mentions_24h: 210,
            unique_accounts_24h: 96,
            engagement_score: 31.0,
            sentiment_net: -0.04,
            bot_probability: 0.22,
            organicness: 0.67,
            narrative_diversity: 0.34,
            dominant_topics: vec!["meme".into(), "nostalgia".into()],
            spam_ratio: 0.11,
            provenance: stamp_now("social", "fixture-v1", 0.52),
            data_state: DataState::Simulated,
        },
        development: DevelopmentSnapshot {
            commits_30d: 1,
            active_contributors_30d: 1,
            releases_90d: 0,
            last_commit_days: Some(190),
            open_issues: 8,
            activity_score: 8.0,
            provenance: stamp_now("dev", "fixture-v1", 0.48),
            data_state: DataState::Simulated,
        },
    }
}

fn generic(def: &TokenDefinition) -> TokenFixtures {
    let seed = hash_id(&def.token_id);
    let mag = 1e-8 + (seed % 1000) as f64 * 1e-12;
    TokenFixtures {
        market: MarketSnapshot {
            price_usd: mag,
            market_cap_usd: 500_000.0 + (seed % 5_000_000) as f64,
            fdv_usd: 800_000.0 + (seed % 8_000_000) as f64,
            volume_24h_usd: 20_000.0 + (seed % 200_000) as f64,
            high_24h: None,
            low_24h: None,
            change_24h_pct: ((seed % 21) as f64) - 10.0,
            ath_usd: None,
            ath_distance_pct: None,
            circulating_supply: def.supply.circulating,
            total_supply: def.supply.total,
            provenance: stamp_now("market", "fixture-v1", 0.4),
            data_state: DataState::Simulated,
        },
        liquidity: LiquiditySnapshot {
            liquidity_usd: 80_000.0 + (seed % 400_000) as f64,
            pool_count: 1 + (seed % 4) as u32,
            spread_bps: 40.0 + (seed % 80) as f64,
            depth_plus_2pct_usd: 8_000.0,
            depth_minus_2pct_usd: 7_500.0,
            lp_change_7d_pct: 0.0,
            buy_sell_imbalance: 0.0,
            provenance: stamp_now("dex", "fixture-v1", 0.4),
            data_state: DataState::Simulated,
            pools: vec![],
            counterparts: vec![],
            official_links: vec![],
        },
        onchain: OnchainSnapshot {
            holders: 1_000 + (seed % 50_000) as u64,
            active_holders_30d: 200 + (seed % 5_000) as u64,
            new_holders_7d: 20,
            retained_holders_30d_pct: 50.0,
            lost_holders_7d: 15,
            whale_holders: 8,
            top10_concentration_pct: 45.0,
            top50_concentration_pct: 70.0,
            transfers_24h: 80,
            unique_senders_24h: 40,
            large_transfers_24h: 1,
            exchange_inflow_usd: 5_000.0,
            exchange_outflow_usd: 4_000.0,
            provenance: stamp_now("onchain", "fixture-v1", 0.4),
            data_state: DataState::Simulated,
            top_holders: vec![],
        },
        social: SocialSnapshot {
            mentions_24h: 40,
            unique_accounts_24h: 20,
            engagement_score: 25.0,
            sentiment_net: 0.0,
            bot_probability: 0.3,
            organicness: 0.5,
            narrative_diversity: 0.3,
            dominant_topics: def.narratives.clone(),
            spam_ratio: 0.2,
            provenance: stamp_now("social", "fixture-v1", 0.35),
            data_state: DataState::Simulated,
        },
        development: DevelopmentSnapshot {
            commits_30d: 0,
            active_contributors_30d: 0,
            releases_90d: 0,
            last_commit_days: None,
            open_issues: 0,
            activity_score: 5.0,
            provenance: stamp_now("dev", "fixture-v1", 0.3),
            data_state: DataState::Simulated,
        },
    }
}

fn hash_id(id: &str) -> u64 {
    let mut h = DefaultHasher::new();
    id.hash(&mut h);
    h.finish()
}

pub fn series(seed: &str, points: usize, base: f64, vol: f64) -> Vec<SparkPoint> {
    let mut h = hash_id(seed) as i64;
    let now = Utc::now();
    let mut v = base;
    (0..points)
        .map(|i| {
            h = h.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let n = ((h.abs() % 1000) as f64 / 1000.0 - 0.5) * vol;
            v = (v * (1.0 + n)).max(base * 0.35);
            let t = now - Duration::hours((points - i) as i64);
            SparkPoint {
                t: t.format("%m-%d %H:%M").to_string(),
                v,
            }
        })
        .collect()
}

pub fn timeline(token_id: &str) -> Vec<TimelineEvent> {
    let now = Utc::now();
    match token_id {
        "arbdoge-ai" => vec![
            ev(now - Duration::hours(7), "liquidity", "Liquidity -6.4% over 7d", Some("-6.4%"), 0.7, "dex"),
            ev(now - Duration::hours(5), "whale", "Whale transfer 2.1% of circulating to CEX", Some("inflow"), 0.74, "onchain"),
            ev(now - Duration::hours(3), "social", "Sentiment reversal on AI narrative", Some("-12 net"), 0.58, "social"),
            ev(now - Duration::hours(1), "volume", "24h volume below 30d median", Some("-18%"), 0.66, "market"),
        ],
        "baby-doge" => vec![
            ev(now - Duration::hours(9), "holders", "Holder base +3.1k net (7d)", Some("+3.1k"), 0.72, "onchain"),
            ev(now - Duration::hours(6), "social", "Organic mentions above 30d average", Some("+14%"), 0.64, "social"),
            ev(now - Duration::hours(2), "development", "Repository activity in last 4 days", None, 0.7, "dev"),
            ev(now - Duration::minutes(50), "liquidity", "Depth stable; spread 22 bps", Some("22bps"), 0.68, "dex"),
        ],
        "kishu-inu" => vec![
            ev(now - Duration::hours(12), "development", "No public commits in 190 days", Some("stale"), 0.8, "dev"),
            ev(now - Duration::hours(8), "onchain", "Active wallets 30d remain subdued", Some("9.6k"), 0.66, "onchain"),
            ev(now - Duration::hours(4), "social", "Narrative diversity compressed to nostalgia/meme", None, 0.55, "social"),
            ev(now - Duration::hours(1), "liquidity", "LP -3.2% over 7 days", Some("-3.2%"), 0.62, "dex"),
        ],
        _ => vec![ev(
            now - Duration::hours(2),
            "index",
            "Token indexed from definition",
            None,
            0.4,
            "registry",
        )],
    }
}

fn ev(
    at: chrono::DateTime<Utc>,
    kind: &str,
    title: &str,
    delta: Option<&str>,
    confidence: f64,
    source: &str,
) -> TimelineEvent {
    TimelineEvent {
        at,
        kind: kind.into(),
        title: title.into(),
        delta: delta.map(|s| s.into()),
        confidence,
        source: source.into(),
    }
}
