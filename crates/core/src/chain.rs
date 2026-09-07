use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Blockchain-agnostic adapter. New chains are added by implementing this
/// trait — never by branching business logic on chain names.
#[async_trait]
pub trait ChainAdapter: Send + Sync {
    fn chain_id(&self) -> &str;
    fn family(&self) -> ChainFamily;
    async fn get_block_height(&self) -> Result<u64>;
    async fn get_token_metadata(&self, contract: &str) -> Result<TokenMetadata>;
    async fn get_balance(&self, wallet: &str, token: &str) -> Result<Balance>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChainFamily {
    Evm,
    Solana,
    Sui,
    Aptos,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub address: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub wallet: String,
    pub token: String,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct EvmAdapter {
    chain_id: String,
    rpc_url: Option<String>,
    client: reqwest::Client,
}

impl EvmAdapter {
    pub fn new(chain_id: impl Into<String>, rpc_url: Option<String>) -> Self {
        Self {
            chain_id: chain_id.into(),
            rpc_url,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build()
                .expect("reqwest client"),
        }
    }

    pub fn rpc_url(&self) -> Option<&str> {
        self.rpc_url.as_deref()
    }

    pub async fn rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let Some(url) = &self.rpc_url else {
            return Err(crate::error::Error::Http("RPC URL not configured".into()));
        };
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        let res = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::error::Error::Http(e.to_string()))?;
        let v: serde_json::Value = res
            .json()
            .await
            .map_err(|e| crate::error::Error::Http(e.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(crate::error::Error::Http(err.to_string()));
        }
        Ok(v.get("result").cloned().unwrap_or(serde_json::Value::Null))
    }
}

#[async_trait]
impl ChainAdapter for EvmAdapter {
    fn chain_id(&self) -> &str {
        &self.chain_id
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Evm
    }

    async fn get_block_height(&self) -> Result<u64> {
        if self.rpc_url.is_none() {
            return Ok(0);
        }
        let result = self.rpc("eth_blockNumber", serde_json::json!([])).await?;
        let hex = result.as_str().unwrap_or("0x0").trim_start_matches("0x");
        u64::from_str_radix(hex, 16).map_err(|e| crate::error::Error::Http(e.to_string()))
    }

    async fn get_token_metadata(&self, contract: &str) -> Result<TokenMetadata> {
        Ok(TokenMetadata {
            address: contract.to_string(),
            name: None,
            symbol: None,
            decimals: None,
        })
    }

    async fn get_balance(&self, wallet: &str, token: &str) -> Result<Balance> {
        Ok(Balance {
            wallet: wallet.to_string(),
            token: token.to_string(),
            raw: "0".into(),
        })
    }
}

/// Base58 mint / account. Never accepts an RPC URL.
pub fn is_solana_mint(addr: &str) -> bool {
    const ALPHA: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let a = addr.trim().as_bytes();
    (32..=44).contains(&a.len()) && a.iter().all(|b| ALPHA.contains(b))
}

/// RPC URL only from `SOLANA_RPC_URL`. Never from a prompt or onboard body.
#[derive(Debug, Clone)]
pub struct SolanaAdapter {
    rpc_url: Option<String>,
    client: reqwest::Client,
}

impl SolanaAdapter {
    pub fn from_env() -> Self {
        Self {
            rpc_url: std::env::var("SOLANA_RPC_URL").ok().filter(|s| !s.is_empty()),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(8))
                .build()
                .expect("reqwest client"),
        }
    }

    pub fn configured(&self) -> bool {
        self.rpc_url.is_some()
    }

    async fn rpc(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let Some(url) = &self.rpc_url else {
            return Err(crate::error::Error::Http("SOLANA_RPC_URL not configured".into()));
        };
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        let res = self
            .client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| crate::error::Error::Http(e.to_string()))?;
        let v: serde_json::Value = res
            .json()
            .await
            .map_err(|e| crate::error::Error::Http(e.to_string()))?;
        if let Some(err) = v.get("error") {
            return Err(crate::error::Error::Http(err.to_string()));
        }
        Ok(v.get("result").cloned().unwrap_or(serde_json::Value::Null))
    }
}

#[async_trait]
impl ChainAdapter for SolanaAdapter {
    fn chain_id(&self) -> &str {
        "solana"
    }

    fn family(&self) -> ChainFamily {
        ChainFamily::Solana
    }

    async fn get_block_height(&self) -> Result<u64> {
        if self.rpc_url.is_none() {
            return Ok(0);
        }
        let slot = self.rpc("getSlot", serde_json::json!([])).await?;
        Ok(slot.as_u64().unwrap_or(0))
    }

    async fn get_token_metadata(&self, contract: &str) -> Result<TokenMetadata> {
        if !is_solana_mint(contract) {
            return Err(crate::error::Error::InvalidDefinition("not a Solana mint".into()));
        }
        Ok(TokenMetadata {
            address: contract.to_string(),
            name: None,
            symbol: None,
            decimals: None,
        })
    }

    async fn get_balance(&self, _wallet: &str, _token: &str) -> Result<Balance> {
        Err(crate::error::Error::Http(
            "Solana balances stay MISSING until an indexer exists — not invented".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solana_mint_rejects_evm() {
        assert!(!is_solana_mint("0x6982508145454Ce325dDbE47a25d4ec3d2311933"));
        assert!(is_solana_mint("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));
    }

    #[test]
    fn solana_adapter_without_env_is_not_live() {
        let a = SolanaAdapter {
            rpc_url: None,
            client: reqwest::Client::new(),
        };
        assert!(!a.configured());
    }
}

pub fn default_evm_adapters() -> Vec<EvmAdapter> {
    vec![
        EvmAdapter::new("ethereum", std::env::var("RPC_ETHEREUM").ok()),
        EvmAdapter::new("arbitrum", std::env::var("RPC_ARBITRUM").ok()),
        EvmAdapter::new("bsc", std::env::var("RPC_BSC").ok()),
        EvmAdapter::new("base", std::env::var("RPC_BASE").ok()),
        EvmAdapter::new("polygon", std::env::var("RPC_POLYGON").ok()),
        EvmAdapter::new("optimism", std::env::var("RPC_OPTIMISM").ok()),
        EvmAdapter::new("avalanche", std::env::var("RPC_AVALANCHE").ok()),
    ]
}
