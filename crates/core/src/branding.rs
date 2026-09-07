use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantBranding {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub custom_domain: Option<String>,
}

/// HTTPS logos only. No JS URLs. Accent is #RRGGBB. Domain is a hostname.
pub fn validate(b: &TenantBranding) -> Result<TenantBranding, String> {
    let mut out = TenantBranding::default();
    if let Some(name) = b.display_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if name.len() > 64 {
            return Err("display_name max 64 chars".into());
        }
        out.display_name = Some(name.to_string());
    }
    if let Some(url) = b.logo_url.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let lower = url.to_ascii_lowercase();
        if !lower.starts_with("https://") {
            return Err("logo_url must be https".into());
        }
        if lower.starts_with("javascript:") || lower.contains("<script") {
            return Err("logo_url rejected".into());
        }
        out.logo_url = Some(url.to_string());
    }
    if let Some(accent) = b.accent.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if !is_hex_color(accent) {
            return Err("accent must be #RRGGBB".into());
        }
        out.accent = Some(accent.to_ascii_uppercase());
    }
    if let Some(dom) = b.custom_domain.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        if !is_hostname(dom) {
            return Err("custom_domain must be a hostname".into());
        }
        out.custom_domain = Some(dom.to_ascii_lowercase());
    }
    Ok(out)
}

fn is_hex_color(s: &str) -> bool {
    let s = s.strip_prefix('#').unwrap_or(s);
    s.len() == 6 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn is_hostname(s: &str) -> bool {
    if s.len() < 3 || s.len() > 253 || s.contains('/') || s.contains(':') || s.contains(' ') {
        return false;
    }
    s.split('.').all(|p| {
        !p.is_empty()
            && p.len() <= 63
            && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            && !p.starts_with('-')
            && !p.ends_with('-')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_http_logo_and_accepts_https() {
        assert!(validate(&TenantBranding {
            logo_url: Some("http://evil.local/x.png".into()),
            ..Default::default()
        })
        .is_err());
        let ok = validate(&TenantBranding {
            display_name: Some("Acme Twin".into()),
            logo_url: Some("https://cdn.example.com/logo.png".into()),
            accent: Some("#3ee58a".into()),
            custom_domain: Some("twin.example.com".into()),
        })
        .unwrap();
        assert_eq!(ok.display_name.as_deref(), Some("Acme Twin"));
        assert_eq!(ok.accent.as_deref(), Some("#3EE58A"));
    }
}
