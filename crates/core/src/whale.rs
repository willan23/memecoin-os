use crate::cex;
use crate::indexer::IndexedTransfer;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WhaleKind {
    WhaleTransfer,
    WhaleAccumulation,
    WhaleDistribution,
    CexDeposit,
    CexWithdrawal,
    LpInteraction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WhaleEvent {
    pub kind: WhaleKind,
    pub direction: String,
    pub tx_hash: String,
    pub chain_id: String,
    pub from_address: String,
    pub to_address: String,
    pub wallet: String,
    pub exchange: Option<String>,
    pub amount_raw: String,
    pub amount_usd: Option<f64>,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Classify a real transfer. Never a buy/sell recommendation.
pub fn classify(
    t: &IndexedTransfer,
    whales: &HashSet<String>,
    pools: &HashSet<String>,
    amount_usd: Option<f64>,
    large: bool,
) -> Vec<WhaleEvent> {
    let from = t.from_address.to_lowercase();
    let to = t.to_address.to_lowercase();
    let from_whale = whales.contains(&from);
    let to_whale = whales.contains(&to);
    let from_cex = cex::lookup(&t.chain_id, &from);
    let to_cex = cex::lookup(&t.chain_id, &to);
    let from_lp = pools.contains(&from);
    let to_lp = pools.contains(&to);

    let mut out = Vec::new();
    let usd_note = amount_usd
        .map(|v| format!("approx ${v:.2} from spot × raw/decimals"))
        .unwrap_or_else(|| "USD not computed".into());

    if from_whale && !to_whale {
        out.push(event(
            t,
            WhaleKind::WhaleDistribution,
            "out",
            from.clone(),
            None,
            amount_usd,
            0.62,
            vec![
                "from address is a ≥1% holder candidate".into(),
                usd_note.clone(),
                "not a sell recommendation".into(),
            ],
        ));
    } else if to_whale && !from_whale {
        out.push(event(
            t,
            WhaleKind::WhaleAccumulation,
            "in",
            to.clone(),
            None,
            amount_usd,
            0.62,
            vec![
                "to address is a ≥1% holder candidate".into(),
                usd_note.clone(),
                "not a buy recommendation".into(),
            ],
        ));
    } else if from_whale || to_whale {
        out.push(event(
            t,
            WhaleKind::WhaleTransfer,
            "between",
            if from_whale { from.clone() } else { to.clone() },
            None,
            amount_usd,
            0.55,
            vec!["whale-to-whale or unclassified counterpart".into(), usd_note.clone()],
        ));
    }

    if let Some(ex) = to_cex {
        out.push(event(
            t,
            WhaleKind::CexDeposit,
            "cex_in",
            to.clone(),
            Some(ex.name.into()),
            amount_usd,
            cex::label_confidence(),
            vec![
                format!("to matches public {} label (incomplete list)", ex.name),
                usd_note.clone(),
            ],
        ));
    }
    if let Some(ex) = from_cex {
        out.push(event(
            t,
            WhaleKind::CexWithdrawal,
            "cex_out",
            from.clone(),
            Some(ex.name.into()),
            amount_usd,
            cex::label_confidence(),
            vec![
                format!("from matches public {} label (incomplete list)", ex.name),
                usd_note.clone(),
            ],
        ));
    }
    if from_lp || to_lp {
        out.push(event(
            t,
            WhaleKind::LpInteraction,
            "lp",
            if to_lp { to } else { from },
            None,
            amount_usd,
            0.6,
            vec!["counterparty is a discovered DEX pair".into()],
        ));
    }

    if large {
        for e in &mut out {
            e.evidence.push("amount is an outlier vs this index window".into());
            e.confidence = (e.confidence + 0.05).min(0.75);
        }
    }
    out
}

fn event(
    t: &IndexedTransfer,
    kind: WhaleKind,
    direction: &str,
    wallet: String,
    exchange: Option<String>,
    amount_usd: Option<f64>,
    confidence: f64,
    evidence: Vec<String>,
) -> WhaleEvent {
    WhaleEvent {
        kind,
        direction: direction.into(),
        tx_hash: t.tx_hash.clone(),
        chain_id: t.chain_id.clone(),
        from_address: t.from_address.clone(),
        to_address: t.to_address.clone(),
        wallet,
        exchange,
        amount_raw: t.amount_raw.clone(),
        amount_usd,
        confidence,
        evidence,
    }
}

/// Indices whose amount is ≥ 2.5σ above the window mean. Need ≥5 present values.
pub fn large_indices(amounts: &[f64]) -> Vec<usize> {
    if amounts.len() < 5 {
        return vec![];
    }
    let n = amounts.len() as f64;
    let mean = amounts.iter().sum::<f64>() / n;
    let var = amounts.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    if std <= 1e-12 {
        return vec![];
    }
    amounts
        .iter()
        .enumerate()
        .filter(|(_, v)| (*v - mean) / std >= 2.5)
        .map(|(i, _)| i)
        .collect()
}

pub fn amount_token_units(amount_raw: &str, decimals: u8) -> Option<f64> {
    let n = if let Some(h) = amount_raw.strip_prefix("0x") {
        u128::from_str_radix(h, 16).ok()?
    } else {
        amount_raw.parse::<u128>().ok()?
    };
    Some((n as f64) / 10f64.powi(decimals as i32))
}

pub fn amount_usd(amount_raw: &str, decimals: u8, price_usd: f64) -> Option<f64> {
    if price_usd <= 0.0 {
        return None;
    }
    Some(amount_token_units(amount_raw, decimals)? * price_usd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::IndexedTransfer;

    fn t(from: &str, to: &str) -> IndexedTransfer {
        IndexedTransfer {
            chain_id: "ethereum".into(),
            tx_hash: "0xabc".into(),
            block_number: 1,
            from_address: from.into(),
            to_address: to.into(),
            token: "0x6982508145454ce325ddbe47a25d4ec3d2311933".into(),
            amount_raw: "1000000000000000000".into(),
        }
    }

    #[test]
    fn whale_to_cex_is_distribution_and_deposit() {
        let whale = "0x1111111111111111111111111111111111111111";
        let binance = "0x28c6c06298d514db089934071355e5743bf21d60";
        let mut whales = HashSet::new();
        whales.insert(whale.into());
        let ev = classify(&t(whale, binance), &whales, &HashSet::new(), Some(12.0), false);
        assert!(ev.iter().any(|e| e.kind == WhaleKind::WhaleDistribution));
        assert!(ev.iter().any(|e| e.kind == WhaleKind::CexDeposit && e.exchange.as_deref() == Some("binance")));
        assert!(ev.iter().all(|e| e.evidence.iter().any(|x| x.contains("not a sell") || x.contains("incomplete") || x.contains("USD"))));
    }

    #[test]
    fn holders_alone_do_not_create_events() {
        let whales = HashSet::from(["0xaaa".into()]);
        assert!(classify(&t("0xbbb", "0xccc"), &whales, &HashSet::new(), None, false).is_empty());
    }

    #[test]
    fn large_indices_need_five_points() {
        assert!(large_indices(&[1.0, 1.0, 100.0]).is_empty());
        let idx = large_indices(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 50.0]);
        assert_eq!(idx, vec![9]);
    }
}
