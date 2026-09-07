use crate::chain::{ChainAdapter, EvmAdapter};
use crate::error::Result;
use crate::models::DiscoveredPool;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// keccak256("Transfer(address,address,uint256)")
pub const ERC20_TRANSFER_TOPIC: &str =
    "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";

/// EVM indexer port. Transfers stay empty until RPC or Etherscan is configured.
/// Never invents transfers from holder counts.
#[async_trait]
pub trait ChainIndexer: Send + Sync {
    fn chain_id(&self) -> &str;
    fn rpc_configured(&self) -> bool;
    async fn get_block_height(&self) -> Result<Option<u64>>;
    async fn index_transfers(
        &self,
        token: &str,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<IndexedTransfer>>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexedTransfer {
    pub chain_id: String,
    pub tx_hash: String,
    pub block_number: u64,
    pub from_address: String,
    pub to_address: String,
    pub token: String,
    pub amount_raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IndexerStatus {
    pub chain_id: String,
    pub rpc_configured: bool,
    pub block_height: Option<u64>,
    pub transfers_indexed: bool,
    pub note: String,
}

#[derive(Debug, Clone)]
pub struct CursorWindow {
    pub from_block: u64,
    pub to_block: u64,
}

pub fn confirmations() -> u64 {
    std::env::var("INDEXER_CONFIRMATIONS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6)
}

pub fn block_span() -> u64 {
    std::env::var("INDEXER_BLOCK_SPAN")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200)
}

/// Advance a cursor toward `tip` without scanning genesis. Huge gaps skip to the tip window.
pub fn next_window(last_block: Option<u64>, tip: u64) -> Option<CursorWindow> {
    let conf = confirmations();
    let span = block_span().max(1);
    if tip <= conf {
        return None;
    }
    let safe_tip = tip - conf;
    let from = match last_block {
        Some(b) if b + 1 <= safe_tip => b + 1,
        Some(_) => return None,
        None => safe_tip.saturating_sub(span - 1),
    };
    let mut from = from.min(safe_tip);
    if safe_tip.saturating_sub(from) + 1 > span {
        from = safe_tip.saturating_sub(span - 1);
    }
    if from > safe_tip {
        return None;
    }
    Some(CursorWindow {
        from_block: from,
        to_block: safe_tip,
    })
}

pub struct EvmRpcIndexer {
    adapter: EvmAdapter,
}

impl EvmRpcIndexer {
    pub fn new(adapter: EvmAdapter) -> Self {
        Self { adapter }
    }

    pub fn for_chain(chain_id: &str) -> Option<Self> {
        crate::chain::default_evm_adapters()
            .into_iter()
            .find(|a| a.chain_id() == chain_id)
            .map(Self::new)
    }

    pub fn from_env() -> Vec<Self> {
        crate::chain::default_evm_adapters()
            .into_iter()
            .map(Self::new)
            .collect()
    }
}

#[async_trait]
impl ChainIndexer for EvmRpcIndexer {
    fn chain_id(&self) -> &str {
        self.adapter.chain_id()
    }

    fn rpc_configured(&self) -> bool {
        self.adapter
            .rpc_url()
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
    }

    async fn get_block_height(&self) -> Result<Option<u64>> {
        if !self.rpc_configured() {
            return Ok(None);
        }
        let h = self.adapter.get_block_height().await?;
        if h == 0 {
            Ok(None)
        } else {
            Ok(Some(h))
        }
    }

    async fn index_transfers(
        &self,
        token: &str,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<IndexedTransfer>> {
        if !self.rpc_configured() || from_block > to_block {
            return Ok(vec![]);
        }
        if !is_evm_address(token) {
            return Ok(vec![]);
        }
        let result = self
            .adapter
            .rpc(
                "eth_getLogs",
                serde_json::json!([{
                    "fromBlock": format!("0x{from_block:x}"),
                    "toBlock": format!("0x{to_block:x}"),
                    "address": token,
                    "topics": [ERC20_TRANSFER_TOPIC]
                }]),
            )
            .await?;
        let logs = result.as_array().cloned().unwrap_or_default();
        Ok(logs
            .iter()
            .filter_map(|log| parse_erc20_transfer_log(self.chain_id(), token, log))
            .collect())
    }
}

pub fn is_evm_address(addr: &str) -> bool {
    let a = addr.trim();
    a.len() == 42 && a.starts_with("0x") && a[2..].bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn hex_to_dec(hex: &str) -> String {
    let h = hex.trim().trim_start_matches("0x").trim_start_matches("0X");
    if h.is_empty() {
        return "0".into();
    }
    u128::from_str_radix(h, 16)
        .map(|n| n.to_string())
        .unwrap_or_else(|_| format!("0x{h}"))
}

pub fn topic_address(topic: &str) -> Option<String> {
    let h = topic.trim().trim_start_matches("0x");
    if h.len() < 40 {
        return None;
    }
    let addr = format!("0x{}", &h[h.len() - 40..]);
    if is_evm_address(&addr) {
        Some(addr.to_lowercase())
    } else {
        None
    }
}

pub fn parse_erc20_transfer_log(
    chain_id: &str,
    token: &str,
    log: &serde_json::Value,
) -> Option<IndexedTransfer> {
    let topics = log.get("topics")?.as_array()?;
    let topic0 = topics.first()?.as_str()?.to_lowercase();
    if topic0 != ERC20_TRANSFER_TOPIC {
        return None;
    }
    let from = topic_address(topics.get(1)?.as_str()?)?;
    let to = topic_address(topics.get(2)?.as_str()?)?;
    let data = log.get("data")?.as_str().unwrap_or("0x0");
    let tx_hash = log.get("transactionHash")?.as_str()?.to_lowercase();
    let block_hex = log.get("blockNumber")?.as_str()?.trim_start_matches("0x");
    let block_number = u64::from_str_radix(block_hex, 16).ok()?;
    Some(IndexedTransfer {
        chain_id: chain_id.into(),
        tx_hash,
        block_number,
        from_address: from,
        to_address: to,
        token: token.to_lowercase(),
        amount_raw: hex_to_dec(data),
    })
}

pub fn parse_etherscan_tokentx(chain_id: &str, token: &str, row: &serde_json::Value) -> Option<IndexedTransfer> {
    let contract = row
        .get("contractAddress")
        .and_then(|v| v.as_str())
        .unwrap_or(token)
        .to_lowercase();
    if contract != token.to_lowercase() {
        return None;
    }
    let tx_hash = row.get("hash")?.as_str()?.to_lowercase();
    let from = row.get("from")?.as_str()?.to_lowercase();
    let to = row.get("to")?.as_str()?.to_lowercase();
    if !is_evm_address(&from) || !is_evm_address(&to) {
        return None;
    }
    let amount_raw = row.get("value")?.as_str()?.to_string();
    let block_number = row
        .get("blockNumber")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())?;
    Some(IndexedTransfer {
        chain_id: chain_id.into(),
        tx_hash,
        block_number,
        from_address: from,
        to_address: to,
        token: contract,
        amount_raw,
    })
}

pub fn evm_numeric_id(chain: &str) -> Option<u64> {
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

pub async fn etherscan_recent_transfers(
    client: &reqwest::Client,
    chain_id: &str,
    token: &str,
    api_key: &str,
) -> Result<Vec<IndexedTransfer>> {
    let Some(cid) = evm_numeric_id(chain_id) else {
        return Ok(vec![]);
    };
    if !is_evm_address(token) {
        return Ok(vec![]);
    }
    let url = format!(
        "https://api.etherscan.io/v2/api?chainid={cid}&module=account&action=tokentx&contractaddress={token}&page=1&offset=100&sort=desc&apikey={api_key}"
    );
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| crate::error::Error::Http(e.to_string()))?;
    if !res.status().is_success() {
        return Err(crate::error::Error::Http(format!(
            "etherscan tokentx {}",
            res.status()
        )));
    }
    let v: serde_json::Value = res
        .json()
        .await
        .map_err(|e| crate::error::Error::Http(e.to_string()))?;
    if v.get("status").and_then(|s| s.as_str()) != Some("1") {
        return Ok(vec![]);
    }
    let rows = v.get("result").and_then(|r| r.as_array()).cloned().unwrap_or_default();
    Ok(rows
        .iter()
        .filter_map(|row| parse_etherscan_tokentx(chain_id, token, row))
        .collect())
}

pub async fn status_all() -> Vec<IndexerStatus> {
    let mut out = Vec::new();
    for ix in EvmRpcIndexer::from_env() {
        let height = ix.get_block_height().await.ok().flatten();
        let configured = ix.rpc_configured();
        out.push(IndexerStatus {
            chain_id: ix.chain_id().to_string(),
            rpc_configured: configured,
            block_height: height,
            transfers_indexed: configured,
            note: if configured {
                "RPC set — cursor advances on refresh when jobs.indexer is on"
            } else {
                "RPC unset — transfers MISSING unless ETHERSCAN_API_KEY is set"
            }
            .into(),
        });
    }
    out
}

/// DexScreener pairs → discovered pools. No invented pair addresses.
pub fn pools_from_dex(
    chain_id: &str,
    pairs: &[(String, Option<String>, Option<f64>, Option<f64>)],
) -> Vec<DiscoveredPool> {
    pairs
        .iter()
        .filter(|(addr, _, _, _)| is_evm_address(addr))
        .map(|(addr, dex, liq, px)| DiscoveredPool {
            chain_id: chain_id.into(),
            pair_address: addr.to_lowercase(),
            dex: dex.clone(),
            liquidity_usd: *liq,
            price_usd: *px,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_pair_addresses() {
        let pools = pools_from_dex(
            "ethereum",
            &[
                ("not-an-address".into(), None, Some(1.0), None),
                (
                    "0x6982508145454Ce325dDbE47a25d4ec3d2311933".into(),
                    Some("uniswap".into()),
                    Some(100.0),
                    Some(1.0),
                ),
            ],
        );
        assert_eq!(pools.len(), 1);
        assert_eq!(pools[0].pair_address.len(), 42);
    }

    #[test]
    fn parses_standard_transfer_log() {
        let log = serde_json::json!({
            "address": "0x6982508145454ce325ddbe47a25d4ec3d2311933",
            "topics": [
                ERC20_TRANSFER_TOPIC,
                "0x0000000000000000000000001111111111111111111111111111111111111111",
                "0x00000000000000000000000028c6c06298d514db089934071355e5743bf21d60"
            ],
            "data": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000",
            "transactionHash": "0xabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabcabca",
            "blockNumber": "0x10"
        });
        let t = parse_erc20_transfer_log(
            "ethereum",
            "0x6982508145454Ce325dDbE47a25d4ec3d2311933",
            &log,
        )
        .unwrap();
        assert_eq!(t.from_address, "0x1111111111111111111111111111111111111111");
        assert_eq!(t.to_address, "0x28c6c06298d514db089934071355e5743bf21d60");
        assert_eq!(t.amount_raw, "1000000000000000000");
        assert_eq!(t.block_number, 16);
    }

    #[test]
    fn cursor_skips_huge_gaps_and_needs_confirmations() {
        let w = next_window(Some(1), 10_000).unwrap();
        assert_eq!(w.to_block, 10_000 - confirmations());
        assert!(w.to_block - w.from_block + 1 <= block_span());
        assert!(next_window(Some(10_000), 10_000).is_none());
    }

    #[test]
    fn holder_count_is_not_a_transfer() {
        assert!(parse_erc20_transfer_log("ethereum", "0x6982508145454ce325ddbe47a25d4ec3d2311933", &serde_json::json!({"holders": 99})).is_none());
    }

    #[tokio::test]
    async fn unset_rpc_reports_missing_height() {
        let ix = EvmRpcIndexer::new(crate::chain::EvmAdapter::new("ethereum", None));
        if !ix.rpc_configured() {
            assert_eq!(ix.get_block_height().await.unwrap(), None);
            assert!(ix
                .index_transfers("0x6982508145454ce325ddbe47a25d4ec3d2311933", 1, 2)
                .await
                .unwrap()
                .is_empty());
        }
    }
}
