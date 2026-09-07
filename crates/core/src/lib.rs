pub mod agents;
pub mod ai;
pub mod alerts;
pub mod evidence;
pub mod forecast;
pub mod research;
pub mod billing;
pub mod branding;
pub mod stripe_connect;
pub mod storage;
pub mod baseline;
pub mod ecosystem;
pub mod twin;
pub mod twin_graph;
pub mod twin_sim;
pub mod twin_similarity;
pub mod twin_replay;
pub mod whale_behaviour;
pub mod claims;
pub mod cex;
pub mod chain;
pub mod cluster;
pub mod discovery;
pub mod domain_events;
pub mod genome_cluster;
pub mod narrative;
pub mod verification;
pub mod whale;
pub mod engine;
pub mod error;
pub mod fixtures;
pub mod growth;
pub mod indexer;
pub mod models;
pub mod providers;
pub mod quality;
pub mod registry;
pub mod risk;
pub mod scoring;
pub mod wallet;

pub use engine::IntelligenceEngine;
pub use error::{Error, Result};
pub use registry::TokenRegistry;

use crate::models::{ChainBinding, ChainContracts, DataSources, FeatureFlags, TokenContract, TokenDefinition, TokenStatus, VerificationLevel};

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OnboardRequest {
    pub name: String,
    pub symbol: String,
    pub chain: String,
    pub address: String,
    pub website: Option<String>,
    pub decimals: Option<u8>,
}

const EVM_CHAINS: &[&str] = &[
    "ethereum",
    "arbitrum",
    "bsc",
    "base",
    "polygon",
    "optimism",
    "avalanche",
];

fn is_evm_address(addr: &str) -> bool {
    let a = addr.trim();
    a.len() == 42
        && a.starts_with("0x")
        && a[2..].bytes().all(|b| b.is_ascii_hexdigit())
}

pub fn onboard_definition(req: OnboardRequest) -> Result<TokenDefinition> {
    let symbol = req.symbol.trim().to_uppercase();
    let name = req.name.trim().to_string();
    if symbol.is_empty() || name.is_empty() {
        return Err(Error::InvalidDefinition("name and symbol are required".into()));
    }
    let chain = req.chain.trim().to_lowercase();
    let (standard, address, decimals) = if chain == "solana" {
        if !crate::chain::is_solana_mint(&req.address) {
            return Err(Error::InvalidDefinition(
                "Solana mint must be a base58 address (32–44 chars). RPC is never taken from this request.".into(),
            ));
        }
        ("spl", req.address.trim().to_string(), req.decimals.unwrap_or(9))
    } else if EVM_CHAINS.contains(&chain.as_str()) {
        if !is_evm_address(&req.address) {
            return Err(Error::InvalidDefinition(
                "contract address must be a 20-byte 0x-prefixed EVM address".into(),
            ));
        }
        if crate::discovery::is_quote_asset(&req.address) || crate::cex::is_hub(&req.address) {
            return Err(Error::InvalidDefinition(
                "address is a quote asset or known CEX hub — not an ecosystem to track".into(),
            ));
        }
        ("erc20", req.address, req.decimals.unwrap_or(18))
    } else {
        return Err(Error::InvalidDefinition(format!(
            "chain `{chain}` is not in the EVM allow-list or solana"
        )));
    };
    let token_id = slug(&symbol);
    Ok(TokenDefinition {
        schema_version: 1,
        token_id,
        symbol,
        name,
        status: TokenStatus::Unverified,
        verification: VerificationLevel::Unverified,
        category: "memecoin".into(),
        narratives: vec!["meme".into()],
        launch_date: None,
        website: req.website,
        description: Some("Onboarded via wizard. Status UNVERIFIED — not a safety rating.".into()),
        chains: vec![ChainBinding {
            id: chain,
            standard: standard.into(),
            is_primary: true,
            contracts: ChainContracts {
                token: TokenContract {
                    address,
                    decimals,
                    explorer: None,
                },
            },
        }],
        socials: Default::default(),
        data_sources: DataSources {
            onchain: true,
            social: true,
            github: false,
            exchanges: true,
            news: false,
            market: None,
        },
        features: FeatureFlags::default(),
        supply: Default::default(),
    })
}

fn slug(symbol: &str) -> String {
    let s = symbol
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect::<String>();
    format!("{}-{}", s, &uuid::Uuid::new_v4().to_string()[..8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_onboard_accepts_pepe_address() {
        let def = onboard_definition(OnboardRequest {
            name: "Pepe".into(),
            symbol: "PEPE".into(),
            chain: "ethereum".into(),
            address: "0x6982508145454Ce325dDbE47a25d4ec3d2311933".into(),
            website: None,
            decimals: Some(18),
        })
        .expect("onboard");
        assert_eq!(def.chains[0].id, "ethereum");
        assert_eq!(def.chains[0].contracts.token.address.len(), 42);
    }

    #[test]
    fn evm_onboard_rejects_short_address_and_unknown_chain() {
        assert!(onboard_definition(OnboardRequest {
            name: "X".into(),
            symbol: "X".into(),
            chain: "ethereum".into(),
            address: "0x1234".into(),
            website: None,
            decimals: None,
        })
        .is_err());
        assert!(onboard_definition(OnboardRequest {
            name: "X".into(),
            symbol: "X".into(),
            chain: "solana".into(),
            address: "0x6982508145454Ce325dDbE47a25d4ec3d2311933".into(),
            website: None,
            decimals: None,
        })
        .is_err());
    }

    #[test]
    fn solana_onboard_accepts_base58_mint() {
        let def = onboard_definition(OnboardRequest {
            name: "Usd Coin".into(),
            symbol: "USDC".into(),
            chain: "solana".into(),
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
            website: None,
            decimals: Some(6),
        })
        .expect("solana onboard");
        assert_eq!(def.chains[0].id, "solana");
        assert_eq!(def.chains[0].standard, "spl");
    }
}
