use crate::models::DataState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Forecast {
    pub metric: String,
    pub horizon: String,
    pub point: Option<f64>,
    pub interval: Option<(f64, f64)>,
    pub model_version: String,
    pub features: Vec<String>,
    pub limitations: Vec<String>,
    pub data_state: DataState,
}

const FORBIDDEN: &[&str] = &["price", "price_usd", "target_price"];

/// Naive last-value ± 1.5σ. Never for price. Insufficient series → MISSING.
pub fn forecast(metric: &str, values: &[f64], horizon: &str) -> Forecast {
    let metric_l = metric.to_lowercase();
    if FORBIDDEN.iter().any(|f| metric_l.contains(f)) {
        return Forecast {
            metric: metric.into(),
            horizon: horizon.into(),
            point: None,
            interval: None,
            model_version: "refuse-price/v1".into(),
            features: vec![],
            limitations: vec!["This system does not forecast price.".into()],
            data_state: DataState::Missing,
        };
    }
    if values.len() < 5 {
        return Forecast {
            metric: metric.into(),
            horizon: horizon.into(),
            point: None,
            interval: None,
            model_version: "naive-baseline/v1".into(),
            features: vec!["last_value".into(), "stddev".into()],
            limitations: vec!["INSUFFICIENT_EVIDENCE — need ≥5 live points".into()],
            data_state: DataState::Missing,
        };
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std = var.sqrt();
    let last = *values.last().unwrap();
    Forecast {
        metric: metric.into(),
        horizon: horizon.into(),
        point: Some(last),
        interval: Some((last - 1.5 * std, last + 1.5 * std)),
        model_version: "naive-baseline/v1".into(),
        features: vec!["last_value".into(), "window_stddev".into()],
        limitations: vec![
            "Not a price forecast.".into(),
            "Naive persistence; not a guarantee.".into(),
        ],
        data_state: DataState::Live,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_price_and_needs_points() {
        let f = forecast("price_usd", &[1.0, 2.0, 3.0, 4.0, 5.0], "7d");
        assert_eq!(f.data_state, DataState::Missing);
        assert!(f.limitations.iter().any(|l| l.contains("does not forecast price")));
        assert!(forecast("liquidity_usd", &[1.0], "7d").point.is_none());
        let ok = forecast("volume_24h_usd", &[10.0, 11.0, 9.0, 10.0, 12.0], "7d");
        assert!(ok.point.is_some());
        assert!(ok.limitations.iter().any(|l| l.contains("Not a price")));
    }
}
