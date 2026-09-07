use crate::models::DataState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BaselineWindow {
    H1,
    H6,
    D1,
    D7,
    D30,
    D90,
}

impl BaselineWindow {
    pub fn as_str(self) -> &'static str {
        match self {
            BaselineWindow::H1 => "1h",
            BaselineWindow::H6 => "6h",
            BaselineWindow::D1 => "24h",
            BaselineWindow::D7 => "7d",
            BaselineWindow::D30 => "30d",
            BaselineWindow::D90 => "90d",
        }
    }

    /// Hours of history this window reads. Used by the persist job to slice series.
    pub fn hours(self) -> i64 {
        match self {
            BaselineWindow::H1 => 1,
            BaselineWindow::H6 => 6,
            BaselineWindow::D1 => 24,
            BaselineWindow::D7 => 24 * 7,
            BaselineWindow::D30 => 24 * 30,
            BaselineWindow::D90 => 24 * 90,
        }
    }

    pub fn min_points(self) -> usize {
        match self {
            BaselineWindow::H1 | BaselineWindow::H6 => 3,
            BaselineWindow::D1 => 4,
            BaselineWindow::D7 | BaselineWindow::D30 => 5,
            BaselineWindow::D90 => 8,
        }
    }

    pub fn all() -> [BaselineWindow; 6] {
        [
            BaselineWindow::H1,
            BaselineWindow::H6,
            BaselineWindow::D1,
            BaselineWindow::D7,
            BaselineWindow::D30,
            BaselineWindow::D90,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricBaseline {
    pub metric: String,
    pub window: String,
    pub n: u32,
    pub mean: f64,
    pub stddev: f64,
    pub last: f64,
    pub z_score: Option<f64>,
    #[serde(default)]
    pub mad: f64,
    #[serde(default)]
    pub percentile_25: Option<f64>,
    #[serde(default)]
    pub percentile_75: Option<f64>,
    pub data_state: DataState,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaselineAnomaly {
    pub metric: String,
    pub window: String,
    pub severity: String,
    pub confidence: f64,
    pub baseline: f64,
    pub observed: f64,
    pub deviation: f64,
    pub evidence: Vec<String>,
}

/// Compute mean/stddev/z/MAD/percentiles from a present numeric series. Empty or tiny series → MISSING.
pub fn compute(metric: &str, window: BaselineWindow, values: &[f64]) -> MetricBaseline {
    let n = values.len();
    if n < window.min_points() {
        return MetricBaseline {
            metric: metric.into(),
            window: window.as_str().into(),
            n: n as u32,
            mean: 0.0,
            stddev: 0.0,
            last: values.last().copied().unwrap_or(0.0),
            z_score: None,
            mad: 0.0,
            percentile_25: None,
            percentile_75: None,
            data_state: DataState::Missing,
            evidence: vec![format!(
                "need ≥{} live points, have {n}",
                window.min_points()
            )],
        };
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let var = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let stddev = var.sqrt();
    let last = *values.last().unwrap();
    let z = if stddev > 1e-12 {
        Some((last - mean) / stddev)
    } else {
        Some(0.0)
    };
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let med = median(&sorted);
    let mut abs_dev: Vec<f64> = values.iter().map(|v| (v - med).abs()).collect();
    abs_dev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mad = median(&abs_dev);
    let p25 = percentile(&sorted, 0.25);
    let p75 = percentile(&sorted, 0.75);
    MetricBaseline {
        metric: metric.into(),
        window: window.as_str().into(),
        n: n as u32,
        mean,
        stddev,
        last,
        z_score: z,
        mad,
        percentile_25: p25,
        percentile_75: p75,
        data_state: DataState::Live,
        evidence: vec![
            format!("n={n}"),
            format!("window={}", window.as_str()),
            format!("mad={mad:.6}"),
        ],
    }
}

fn median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return 0.0;
    }
    if n % 2 == 1 {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

fn percentile(sorted: &[f64], p: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    Some(sorted[idx.min(sorted.len() - 1)])
}

const Z_ALERT: f64 = 2.5;

pub fn anomalies(baselines: &[MetricBaseline]) -> Vec<BaselineAnomaly> {
    let mut out = Vec::new();
    for b in baselines {
        if !b.data_state.present() {
            continue;
        }
        let Some(z) = b.z_score else { continue };
        if z.abs() < Z_ALERT {
            continue;
        }
        out.push(BaselineAnomaly {
            metric: b.metric.clone(),
            window: b.window.clone(),
            severity: if z.abs() >= 4.0 { "high" } else { "medium" }.into(),
            confidence: (0.55 + (z.abs() - Z_ALERT) * 0.08).min(0.9),
            baseline: b.mean,
            observed: b.last,
            deviation: z,
            evidence: b.evidence.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_when_too_few_points() {
        let b = compute("price_usd", BaselineWindow::D7, &[1.0, 1.1]);
        assert_eq!(b.data_state, DataState::Missing);
        assert!(anomalies(&[b]).is_empty());
    }

    #[test]
    fn z_score_flags_outlier() {
        let values = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 20.0];
        let b = compute("liquidity_usd", BaselineWindow::D7, &values);
        assert_eq!(b.data_state, DataState::Live);
        let a = anomalies(&[b]);
        assert_eq!(a.len(), 1);
        assert!(a[0].deviation > 2.5);
    }

    #[test]
    fn flat_series_is_not_an_anomaly() {
        let values = vec![5.0; 8];
        let b = compute("volume_24h_usd", BaselineWindow::D7, &values);
        assert_eq!(b.z_score, Some(0.0));
        assert_eq!(b.mad, 0.0);
        assert!(anomalies(&[b]).is_empty());
    }

    #[test]
    fn mad_and_percentiles_on_known_series() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = compute("price_usd", BaselineWindow::D7, &values);
        assert_eq!(b.data_state, DataState::Live);
        assert!((b.mad - 2.0).abs() < 1e-9);
        assert_eq!(b.percentile_25, Some(3.0));
        assert_eq!(b.percentile_75, Some(6.0));
        assert_eq!(BaselineWindow::D90.as_str(), "90d");
        assert_eq!(BaselineWindow::H1.hours(), 1);
        assert_eq!(BaselineWindow::D90.hours(), 24 * 90);
    }
}
