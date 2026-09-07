use crate::error::{Error, Result};
use crate::models::TokenDefinition;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct TokenRegistry {
    tokens: HashMap<String, TokenDefinition>,
    path: PathBuf,
}

impl TokenRegistry {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(Error::InvalidDefinition(format!(
                "token registry path does not exist: {}",
                path.display()
            )));
        }
        let mut tokens = HashMap::new();
        for entry in std::fs::read_dir(&path)? {
            let entry = entry?;
            let file_path = entry.path();
            let ext = file_path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if ext != "yaml" && ext != "yml" {
                continue;
            }
            let raw = std::fs::read_to_string(&file_path)?;
            let def: TokenDefinition = serde_yaml::from_str(&raw)?;
            validate(&def)?;
            tokens.insert(def.token_id.clone(), def);
        }
        if tokens.is_empty() {
            return Err(Error::InvalidDefinition(
                "no token definitions found".into(),
            ));
        }
        Ok(Self { tokens, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn get(&self, id: &str) -> Result<&TokenDefinition> {
        self.tokens
            .get(id)
            .ok_or_else(|| Error::TokenNotFound(id.to_string()))
    }

    pub fn list(&self) -> Vec<&TokenDefinition> {
        let mut items: Vec<_> = self.tokens.values().collect();
        items.sort_by(|a, b| a.token_id.cmp(&b.token_id));
        items
    }

    pub fn insert_runtime(&mut self, def: TokenDefinition) -> Result<()> {
        validate(&def)?;
        // Cloud Run image FS is ephemeral; Postgres `tokens.definition` is the durable copy.
        let _ = self.persist_yaml(&def);
        self.tokens.insert(def.token_id.clone(), def);
        Ok(())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.tokens.contains_key(id)
    }

    pub fn persist_yaml(&self, def: &TokenDefinition) -> Result<()> {
        if self.path.as_os_str().is_empty() {
            return Ok(());
        }
        std::fs::create_dir_all(&self.path)?;
        let path = self.path.join(format!("{}.yaml", def.token_id));
        let yaml = serde_yaml::to_string(def)?;
        std::fs::write(path, yaml)?;
        Ok(())
    }
}

fn validate(def: &TokenDefinition) -> Result<()> {
    if def.token_id.trim().is_empty() {
        return Err(Error::InvalidDefinition("token_id is required".into()));
    }
    if def.symbol.trim().is_empty() {
        return Err(Error::InvalidDefinition("symbol is required".into()));
    }
    if def.chains.is_empty() {
        return Err(Error::InvalidDefinition(
            "at least one chain binding is required".into(),
        ));
    }
    for chain in &def.chains {
        if chain.contracts.token.address.trim().is_empty() {
            return Err(Error::InvalidDefinition(format!(
                "missing token contract on chain {}",
                chain.id
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_mvp_tokens() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tokens");
        let reg = TokenRegistry::load(path).expect("registry");
        assert!(reg.get("arbdoge-ai").is_ok());
        assert!(reg.get("baby-doge").is_ok());
        assert!(reg.get("kishu-inu").is_ok());
        assert!(reg.get("pepe").is_ok());
        assert!(reg.list().len() >= 4);
    }

    #[test]
    fn pepe_yaml_roundtrip_fields() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tokens");
        let reg = TokenRegistry::load(path).expect("registry");
        let pepe = reg.get("pepe").expect("pepe");
        assert_eq!(pepe.symbol, "PEPE");
        assert_eq!(
            pepe.chains[0].contracts.token.address.to_lowercase(),
            "0x6982508145454ce325ddbe47a25d4ec3d2311933"
        );
    }
}
