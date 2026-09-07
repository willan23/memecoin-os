use crate::indexer::is_evm_address;
use crate::models::{TokenDefinition, TokenSnapshot, VerificationLevel};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationReport {
    pub token_id: String,
    pub level: VerificationLevel,
    pub declared_level: VerificationLevel,
    pub reasons: Vec<String>,
    pub evidence: Vec<String>,
    pub last_verified_at: DateTime<Utc>,
    pub disclaimer: String,
}

/// Evidence-based level. Growth never upgrades to community/ecosystem alone.
/// VERIFIED ≠ SAFE.
pub fn assess(def: &TokenDefinition, snap: &TokenSnapshot) -> VerificationReport {
    let now = Utc::now();
    let mut reasons = Vec::new();
    let mut evidence = Vec::new();

    let market_ok = snap.market.data_state.present();
    let liq_ok = snap.liquidity.data_state.present();
    let onchain_ok = snap.onchain.data_state.present() && snap.onchain.holders > 0;
    let contract = def
        .primary_chain()
        .map(|c| c.contracts.token.address.clone())
        .filter(|a| is_evm_address(a));
    let pools = snap.liquidity.pool_count;
    let dev_ok = snap.development.data_state.present();

    if market_ok {
        evidence.push(format!("market {}", snap.market.data_state.as_str()));
    }
    if liq_ok {
        evidence.push(format!(
            "liquidity {} pools={}",
            snap.liquidity.data_state.as_str(),
            pools
        ));
    }
    if onchain_ok {
        evidence.push(format!("holders={}", snap.onchain.holders));
    }
    if let Some(addr) = &contract {
        evidence.push(format!("contract {addr}"));
    }
    if dev_ok {
        evidence.push(format!("github {}", snap.development.data_state.as_str()));
    }

    let level = level_from_flags(
        market_ok,
        liq_ok,
        onchain_ok,
        contract.is_some(),
        pools,
        dev_ok,
        &mut reasons,
    );

    if snap.market.data_state.present() && snap.market.change_24h_pct.abs() > 20.0 {
        evidence.push("price move is not a verification upgrade".into());
    }

    VerificationReport {
        token_id: def.token_id.clone(),
        level,
        declared_level: def.verification.clone(),
        reasons,
        evidence,
        last_verified_at: now,
        disclaimer: "VERIFIED ≠ SAFE. Computed from public evidence, not a risk rating.".into(),
    }
}

pub fn level_from_flags(
    market_ok: bool,
    liq_ok: bool,
    onchain_ok: bool,
    has_contract: bool,
    pools: u32,
    dev_ok: bool,
    reasons: &mut Vec<String>,
) -> VerificationLevel {
    let data_ok = market_ok && liq_ok;
    let contract_ok = data_ok && has_contract && onchain_ok;
    let ecosystem_ok = contract_ok && (pools >= 2 || dev_ok);
    if ecosystem_ok {
        reasons.push("live market + liquidity + holders + (multi-pool or public repo)".into());
        VerificationLevel::EcosystemVerified
    } else if contract_ok {
        reasons.push("live market + liquidity + on-chain holders + valid contract".into());
        VerificationLevel::ContractVerified
    } else if data_ok {
        reasons.push("live market and liquidity present".into());
        VerificationLevel::DataVerified
    } else {
        reasons.push("insufficient live evidence".into());
        VerificationLevel::Unverified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_and_price_cannot_community_verify() {
        let mut reasons = Vec::new();
        let level = level_from_flags(true, true, true, true, 8, true, &mut reasons);
        assert_ne!(level, VerificationLevel::CommunityVerified);
        assert_eq!(level, VerificationLevel::EcosystemVerified);
        let mut reasons2 = Vec::new();
        assert_eq!(
            level_from_flags(false, false, false, false, 0, false, &mut reasons2),
            VerificationLevel::Unverified
        );
    }
}
