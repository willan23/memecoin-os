use axum::http::HeaderMap;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const PREFIX_LEN: usize = 12;

pub fn hash_secret(secret: &str) -> String {
    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    hex::encode(h.finalize())
}

pub fn generate_key() -> (String, String, String) {
    let raw = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    let plaintext = format!("mcos_{raw}");
    let prefix = plaintext.chars().take(PREFIX_LEN).collect::<String>();
    let hash = hash_secret(&plaintext);
    (plaintext, prefix, hash)
}

pub fn prefix_of(presented: &str) -> Option<String> {
    let t = presented.trim();
    if !t.starts_with("mcos_") || t.len() < PREFIX_LEN {
        return None;
    }
    Some(t.chars().take(PREFIX_LEN).collect())
}

pub fn auth_required() -> bool {
    match std::env::var("AUTH_REQUIRED") {
        Ok(v) => matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => false,
    }
}

/// When `OPERATOR_API_KEY` is set, catalog writes need that key. Unset = local open.
pub fn operator_key_configured() -> bool {
    std::env::var("OPERATOR_API_KEY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

pub fn operator_write_allowed(headers: &HeaderMap) -> bool {
    let Ok(expected) = std::env::var("OPERATOR_API_KEY") else {
        return true;
    };
    let expected = expected.trim();
    if expected.is_empty() {
        return true;
    }
    let from_header = headers
        .get("x-operator-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .trim();
    let from_bearer = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("")
        .trim();
    constant_eq(from_header, expected) || constant_eq(from_bearer, expected)
}

fn constant_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub fn deny_unless_operator(headers: &HeaderMap) -> Result<(), (axum::http::StatusCode, String)> {
    if operator_write_allowed(headers) {
        Ok(())
    } else {
        Err((
            axum::http::StatusCode::UNAUTHORIZED,
            "operator key required".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_hashes_and_prefixes() {
        let (plain, prefix, hash) = generate_key();
        assert!(plain.starts_with("mcos_"));
        assert_eq!(prefix, prefix_of(&plain).unwrap());
        assert_eq!(hash, hash_secret(&plain));
        assert_ne!(hash_secret(&plain), hash_secret("other"));
    }

    #[test]
    fn operator_writes_open_when_key_unset() {
        let headers = HeaderMap::new();
        // CI / local without OPERATOR_API_KEY stays writable.
        if !operator_key_configured() {
            assert!(operator_write_allowed(&headers));
        }
    }
}
