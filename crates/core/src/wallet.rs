use serde::{Deserialize, Serialize};

/// Evidence-based wallet class. Never an identity claim.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WalletClassification {
    Unknown,
    Retail,
    Whale,
    Deployer,
    Team,
    Treasury,
    LiquidityProvider,
    MarketMaker,
    Cex,
    Bridge,
    Contract,
    SmartMoneyCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalletProfile {
    pub address: String,
    pub chain_id: String,
    pub classification: WalletClassification,
    pub confidence: f64,
    pub evidence: Vec<String>,
    pub token_id: Option<String>,
    pub share_pct: Option<f64>,
}

/// Classify from public share only. ≥1% of supply → whale candidate, not a person.
pub fn classify_from_share(address: &str, chain_id: &str, token_id: &str, share_pct: f64) -> WalletProfile {
    let (classification, confidence, reason) = if share_pct >= 1.0 {
        (
            WalletClassification::Whale,
            (0.55 + (share_pct / 100.0).min(0.25)).min(0.8),
            format!("on-chain share {share_pct:.2}% ≥ 1% — whale candidate, not an identity"),
        )
    } else if share_pct > 0.0 {
        (
            WalletClassification::Retail,
            0.4,
            format!("on-chain share {share_pct:.2}% < 1%"),
        )
    } else {
        (
            WalletClassification::Unknown,
            0.0,
            "no share evidence".into(),
        )
    };
    WalletProfile {
        address: address.to_lowercase(),
        chain_id: chain_id.into(),
        classification,
        confidence,
        evidence: vec![reason],
        token_id: Some(token_id.into()),
        share_pct: Some(share_pct),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_percent_is_whale_candidate_not_identity() {
        let p = classify_from_share("0xAbc", "ethereum", "pepe", 2.5);
        assert_eq!(p.classification, WalletClassification::Whale);
        assert!(p.evidence.iter().any(|e| e.contains("not an identity")));
        assert_eq!(p.address, "0xabc");
    }

    #[test]
    fn zero_share_is_unknown() {
        let p = classify_from_share("0x1", "ethereum", "pepe", 0.0);
        assert_eq!(p.classification, WalletClassification::Unknown);
        assert_eq!(p.confidence, 0.0);
    }
}
