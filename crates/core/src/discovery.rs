use crate::cex;
use crate::indexer::is_evm_address;
use crate::models::{DiscoveryCandidate, TokenStatus};
use serde::{Deserialize, Serialize};

/// Known quote/wrapper assets — counterparts of these are not "new ecosystems".
const QUOTE_DENY: &[&str] = &[
    "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", // WETH
    "0x0000000000000000000000000000000000000000",
    "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
    "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
    "0x6b175474e89094c44da98b954eedeac495271d0f", // DAI
    "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599", // WBTC
    "0xbb4cdb9cbd36b01bd1cbaebf2de08d9173bc095c", // WBNB
    "0x55d398326f99059ff775485246999027b3197955", // USDT BSC
    "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d", // USDC BSC
    "0x4200000000000000000000000000000000000006", // WETH Base/OP
    "0x82af49447d8a07e3bd95bd0d56f35241523fbab1", // WETH Arb
    "0xe9e7cea3dedca5984780bafc599bd69add087d56", // BUSD BSC
    "0x1af3f329e8be154074d8769d1ffa4ee058b1dbc3", // DAI BSC
    "0x7130d2a12b9bcbfae4f2634d864a1ee1ce3ead9c", // BTCB
    "0x0e09fabb73bd3ade0a17ecc321fd13a19e81ce82", // Cake
    "0x2f2a2543b76a4166549f7aab2e75bef0aefc5b0f", // WBTC Arb
    "0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9", // USD₮0 Arb
    "0xaf88d065e77c8cc2239327c5edb3a432268e5831", // USDC Arb
];

pub fn is_quote_asset(address: &str) -> bool {
    QUOTE_DENY.contains(&address.to_lowercase().as_str())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiscoveryAssessment {
    pub chain_id: String,
    pub address: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub pair_address: Option<String>,
    pub dex: Option<String>,
    pub liquidity_usd: Option<f64>,
    pub via_token: String,
    pub status: TokenStatus,
    pub score: f64,
    pub evidence: Vec<String>,
    pub auto_verified: bool,
}

/// Assess a Dex counterpart. Never returns auto_verified = true.
pub fn assess(c: &DiscoveryCandidate, known_addresses: &[String]) -> Option<DiscoveryAssessment> {
    let address = c.address.to_lowercase();
    if !is_evm_address(&address) || cex::is_hub(&address) || is_quote_asset(&address) {
        return None;
    }
    if known_addresses.iter().any(|k| k.to_lowercase() == address) {
        return None;
    }

    let mut score: f64 = 0.35;
    let mut evidence = vec![format!(
        "seen as pair counterpart of {} on {}",
        c.via_token, c.chain_id
    )];
    if c.symbol.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
        score += 0.15;
        evidence.push(format!("symbol {}", c.symbol.as_deref().unwrap()));
    }
    if c.name.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
        score += 0.1;
        evidence.push("name present on DexScreener pair".into());
    }
    if let Some(liq) = c.liquidity_usd {
        if liq > 0.0 {
            score += 0.1;
            evidence.push(format!("discovering pool liquidity_usd={liq:.0}"));
        }
        if liq >= 50_000.0 {
            score += 0.15;
        }
    }
    score = score.min(0.85);

    let has_meta = c.symbol.is_some() && c.name.is_some();
    let status = if !has_meta {
        TokenStatus::HighRisk
    } else if c.liquidity_usd.unwrap_or(0.0) >= 50_000.0 {
        TokenStatus::Watchlist
    } else {
        TokenStatus::Discovered
    };
    evidence.push("VERIFIED is never assigned by discovery or growth".into());

    Some(DiscoveryAssessment {
        chain_id: c.chain_id.clone(),
        address,
        symbol: c.symbol.clone(),
        name: c.name.clone(),
        pair_address: c.pair_address.clone(),
        dex: c.dex.clone(),
        liquidity_usd: c.liquidity_usd,
        via_token: c.via_token.clone(),
        status,
        score,
        evidence,
        auto_verified: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(addr: &str, symbol: Option<&str>, liq: Option<f64>) -> DiscoveryCandidate {
        DiscoveryCandidate {
            chain_id: "ethereum".into(),
            address: addr.into(),
            symbol: symbol.map(|s| s.into()),
            name: symbol.map(|s| s.into()),
            pair_address: Some("0x6982508145454ce325ddbe47a25d4ec3d2311933".into()),
            dex: Some("uniswap".into()),
            liquidity_usd: liq,
            via_token: "0x6982508145454ce325ddbe47a25d4ec3d2311933".into(),
        }
    }

    #[test]
    fn weth_is_not_discovered() {
        assert!(assess(
            &cand("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", Some("WETH"), Some(1e9)),
            &[]
        )
        .is_none());
    }

    #[test]
    fn growth_does_not_verify() {
        let a = assess(
            &cand("0x1111111111111111111111111111111111111111", Some("FOO"), Some(9e6)),
            &[],
        )
        .unwrap();
        assert!(!a.auto_verified);
        assert_ne!(a.status, TokenStatus::Listed);
        assert_eq!(a.status, TokenStatus::Watchlist);
    }

    #[test]
    fn known_registry_address_skipped() {
        let addr = "0x1111111111111111111111111111111111111111";
        assert!(assess(&cand(addr, Some("FOO"), Some(1.0)), &[addr.into()]).is_none());
    }
}
