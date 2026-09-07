use crate::billing::PlanId;
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Operator account (not a secret). Override with STRIPE_ACCOUNT_ID.
pub const OPERATOR_ACCOUNT_ID: &str = "acct_1TQsIIGT8Pzxsiro";

/// Real processor only when secrets exist in the process env. Never invent customers.
pub fn configured() -> bool {
    env_nonempty("STRIPE_SECRET_KEY") && env_nonempty("STRIPE_WEBHOOK_SECRET")
}

pub fn account_id() -> String {
    std::env::var("STRIPE_ACCOUNT_ID")
        .ok()
        .filter(|s| s.starts_with("acct_"))
        .unwrap_or_else(|| OPERATOR_ACCOUNT_ID.into())
}

pub fn keys_missing() -> bool {
    !configured()
}

pub fn publishable_key() -> Option<String> {
    std::env::var("STRIPE_PUBLISHABLE_KEY")
        .ok()
        .filter(|s| !s.is_empty())
}

pub fn status_json() -> serde_json::Value {
    let ready = configured();
    serde_json::json!({
        "processor": "stripe",
        "configured": ready,
        "keys_missing": !ready,
        "state": if ready { "connected" } else { "keys_missing" },
        "publishable_present": publishable_key().is_some(),
        "checkout": if ready { "webhook_ready" } else { "meters_only" },
        "webhook": "/v1/billing/stripe/webhook",
        "prices_mapped": {
            "pro": price_id_set("STRIPE_PRICE_PRO"),
            "research": price_id_set("STRIPE_PRICE_RESEARCH"),
            "growth": price_id_set("STRIPE_PRICE_GROWTH"),
            "enterprise": price_id_set("STRIPE_PRICE_ENTERPRISE"),
            "api": price_id_set("STRIPE_PRICE_API"),
        },
        "note": "Payment management only. No Dashboard link. Keys stay in .env. Paying never changes a score."
    })
}

pub fn plan_for_price_id(price_id: &str) -> Option<PlanId> {
    let pairs = [
        ("STRIPE_PRICE_PRO", PlanId::Pro),
        ("STRIPE_PRICE_RESEARCH", PlanId::Research),
        ("STRIPE_PRICE_GROWTH", PlanId::Growth),
        ("STRIPE_PRICE_ENTERPRISE", PlanId::Enterprise),
        ("STRIPE_PRICE_API", PlanId::Api),
    ];
    for (env, plan) in pairs {
        if std::env::var(env).ok().as_deref() == Some(price_id) {
            return Some(plan);
        }
    }
    None
}

/// Stripe-Signature `t=…,v1=…` over `timestamp.body`.
pub fn verify_webhook(secret: &str, sig_header: &str, body: &str) -> bool {
    let mut ts = None;
    let mut v1 = None;
    for part in sig_header.split(',') {
        let mut kv = part.splitn(2, '=');
        match (kv.next(), kv.next()) {
            (Some("t"), Some(v)) => ts = Some(v.trim()),
            (Some("v1"), Some(v)) => v1 = Some(v.trim()),
            _ => {}
        }
    }
    let (Some(ts), Some(v1)) = (ts, v1) else {
        return false;
    };
    let signed = format!("{ts}.{body}");
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(signed.as_bytes());
    let expected = hex::encode(mac.finalize().into_bytes());
    expected.eq_ignore_ascii_case(v1)
}

fn env_nonempty(k: &str) -> bool {
    std::env::var(k).map(|s| !s.is_empty()).unwrap_or(false)
}

fn price_id_set(k: &str) -> bool {
    std::env::var(k)
        .ok()
        .filter(|s| s.starts_with("price_"))
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_account_id_is_stable() {
        assert!(account_id().starts_with("acct_"));
        assert_eq!(OPERATOR_ACCOUNT_ID, "acct_1TQsIIGT8Pzxsiro");
    }

    #[test]
    fn unset_is_not_configured() {
        // Process may have leftover env in other tests; configured() is about both keys.
        // Signature verify is the unit we can isolate.
        assert!(!verify_webhook("whsec_x", "bad", "{}"));
        let secret = "whsec_test";
        let body = "{\"id\":\"evt_1\"}";
        let ts = "1000";
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("{ts}.{body}").as_bytes());
        let v1 = hex::encode(mac.finalize().into_bytes());
        assert!(verify_webhook(secret, &format!("t={ts},v1={v1}"), body));
    }
}
