use serde::{Deserialize, Serialize};

/// Public, incomplete CEX/hot-wallet labels. Not exhaustive. Not identity.
/// Addresses are lowercase 0x. Confidence must stay capped — labels can go stale.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct CexLabel {
    pub name: &'static str,
    pub chain_id: &'static str,
}

const ZERO: &str = "0x0000000000000000000000000000000000000000";
const DEAD: &str = "0x000000000000000000000000000000000000dead";

/// Well-known public deposit/hot wallets (Ethereum / BSC). Source: explorer labels.
const CEX: &[(&str, CexLabel)] = &[
    ("0x28c6c06298d514db089934071355e5743bf21d60", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0x21a31ee1afc51d94c2efccaa2092ad1028285549", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0xdfd5293d8e347dfe59e90efd55b2956a1343963d", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0x56eddb7aa87536c09ccc2793473599fd21a8b17f", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0x9696f59e4d72e237be84ffd425dcad154bf96976", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0xbe0eb53f46cd790cd13851d5eff43d12404d33e8", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0x3f5ce5fbfe3e9af3971dd833d26ba9b5c936f0be", CexLabel { name: "binance", chain_id: "ethereum" }),
    ("0x71660c4005ba85c37ccec55d0c4493e66fe775d3", CexLabel { name: "coinbase", chain_id: "ethereum" }),
    ("0x503828976d22510aad0201ac7ec88293211d5230", CexLabel { name: "coinbase", chain_id: "ethereum" }),
    ("0xddfab9e4a04ff3c309092567bae9839f81569eef", CexLabel { name: "coinbase", chain_id: "ethereum" }),
    ("0x3cd751e6b0078be393132286c442345e5dc49699", CexLabel { name: "coinbase", chain_id: "ethereum" }),
    ("0xb5d85cbf7cb3ee0d56b3bb207d5fc4b82f43f511", CexLabel { name: "coinbase", chain_id: "ethereum" }),
    ("0xa090e606e30bd747d4e6245a1517ebe430f0057e", CexLabel { name: "coinbase", chain_id: "ethereum" }),
    ("0x2910543af36e9deed30c7aba451abc55b14ae310", CexLabel { name: "kraken", chain_id: "ethereum" }),
    ("0x0a869d79a7052c7f1b55a8ebabbea3420f0d1e13", CexLabel { name: "kraken", chain_id: "ethereum" }),
    ("0xe853c56864a2ebe4576a807d26fdc4a0ada51919", CexLabel { name: "kraken", chain_id: "ethereum" }),
    ("0x267be1c1d684f78cb4f6a176c4911b741e4ffdc0", CexLabel { name: "kraken", chain_id: "ethereum" }),
    ("0x1151314c646ce4e0efd76d1af4760ae66a9fe30f", CexLabel { name: "bitfinex", chain_id: "ethereum" }),
    ("0x742d35cc6634c0532925a3b844bc454e4438f44e", CexLabel { name: "bitfinex", chain_id: "ethereum" }),
    ("0x876eabf441b2ee5b5b0554fd502a8e0600950cfa", CexLabel { name: "bitfinex", chain_id: "ethereum" }),
    ("0x0681d8db095565fe51a5266e0e5312e4e45b8dc4", CexLabel { name: "okx", chain_id: "ethereum" }),
    ("0x6cc5f688a315f3dc28a7781717a9a798a59fda7b", CexLabel { name: "okx", chain_id: "ethereum" }),
    ("0xf89d7b9c864f589bbf53a82105107622b35eaa40", CexLabel { name: "bybit", chain_id: "ethereum" }),
    ("0x1db92e2eebc8e0c075a02bea49a2935bcd2dfcf4", CexLabel { name: "bybit", chain_id: "ethereum" }),
    ("0x8894e0a0c962cb723c1976a4421c95949be2d4e3", CexLabel { name: "binance", chain_id: "bsc" }),
];

const MAX_CONFIDENCE: f64 = 0.68;

pub fn lookup(chain_id: &str, address: &str) -> Option<CexLabel> {
    let addr = address.to_lowercase();
    CEX.iter().find(|(a, l)| *a == addr && l.chain_id == chain_id).map(|(_, l)| *l)
}

pub fn is_hub(address: &str) -> bool {
    let a = address.to_lowercase();
    a == ZERO || a == DEAD || CEX.iter().any(|(listed, _)| *listed == a)
}

pub fn label_confidence() -> f64 {
    MAX_CONFIDENCE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_binance_is_cex_on_ethereum_only() {
        let a = "0x28C6c06298d514Db089934071355E5743bf21d60";
        assert_eq!(lookup("ethereum", a).map(|l| l.name), Some("binance"));
        assert!(lookup("arbitrum", a).is_none());
        assert!(is_hub(a));
    }

    #[test]
    fn zero_and_dead_are_hubs() {
        assert!(is_hub(ZERO));
        assert!(is_hub("0x000000000000000000000000000000000000dEaD"));
    }
}
