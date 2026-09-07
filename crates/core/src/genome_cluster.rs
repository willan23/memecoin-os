use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct GenomeToken {
    pub token_id: String,
    pub symbol: String,
    pub chain: String,
    pub narratives: Vec<String>,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenomeCluster {
    pub cluster_id: String,
    pub label: String,
    pub token_ids: Vec<String>,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Cluster by genome cosine + narrative Jaccard + chain. Name is not a feature.
pub fn cluster(items: &[GenomeToken], min_sim: f64) -> Vec<GenomeCluster> {
    if items.len() < 2 {
        return vec![];
    }
    let mut assigned = vec![false; items.len()];
    let mut out = Vec::new();
    for i in 0..items.len() {
        if assigned[i] {
            continue;
        }
        let mut group = vec![i];
        assigned[i] = true;
        for j in (i + 1)..items.len() {
            if assigned[j] {
                continue;
            }
            if similarity(&items[i], &items[j]) >= min_sim {
                assigned[j] = true;
                group.push(j);
            }
        }
        if group.len() < 2 {
            assigned[i] = false;
            continue;
        }
        let token_ids: Vec<String> = group.iter().map(|&k| items[k].token_id.clone()).collect();
        let label = majority_narrative(&group.iter().map(|&k| &items[k]).collect::<Vec<_>>())
            .unwrap_or_else(|| "mixed-genome".into());
        let avg: f64 = group
            .iter()
            .skip(1)
            .map(|&k| similarity(&items[group[0]], &items[k]))
            .sum::<f64>()
            / (group.len() - 1) as f64;
        out.push(GenomeCluster {
            cluster_id: format!("g:{label}:{}", token_ids.join("+")),
            label,
            token_ids,
            confidence: avg.min(0.8),
            evidence: vec![
                "features = genome vector + narrative overlap + chain".into(),
                "token name is not used".into(),
            ],
        });
    }
    out.sort_by(|a, b| b.token_ids.len().cmp(&a.token_ids.len()));
    out
}

pub fn similarity(a: &GenomeToken, b: &GenomeToken) -> f64 {
    let cos = cosine(&a.values, &b.values);
    let jac = jaccard(&a.narratives, &b.narratives);
    let chain = if a.chain == b.chain { 1.0 } else { 0.0 };
    0.7 * cos + 0.2 * jac + 0.1 * chain
}

fn cosine(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut dot = 0.0;
    let mut na = 0.0;
    let mut nb = 0.0;
    for i in 0..n {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na <= 1e-12 || nb <= 1e-12 {
        0.0
    } else {
        (dot / (na.sqrt() * nb.sqrt())).clamp(0.0, 1.0)
    }
}

fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let inter = a.iter().filter(|x| b.contains(x)).count() as f64;
    let mut union: Vec<&String> = a.iter().chain(b.iter()).collect();
    union.sort();
    union.dedup();
    inter / union.len() as f64
}

fn majority_narrative(items: &[&GenomeToken]) -> Option<String> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for t in items {
        for n in &t.narratives {
            if let Some((_, c)) = counts.iter_mut().find(|(k, _)| k == n) {
                *c += 1;
            } else {
                counts.push((n.clone(), 1));
            }
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts.into_iter().map(|(k, _)| k).next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(id: &str, chain: &str, tags: &[&str], values: Vec<f64>) -> GenomeToken {
        GenomeToken {
            token_id: id.into(),
            symbol: id.to_uppercase(),
            chain: chain.into(),
            narratives: tags.iter().map(|s| s.to_string()).collect(),
            values,
        }
    }

    #[test]
    fn similar_genomes_cluster_without_using_name() {
        let items = vec![
            tok("alpha-inu", "ethereum", &["meme"], vec![50.0; 8]),
            tok("beta-cat", "ethereum", &["meme"], vec![51.0; 8]),
            tok("gamma-util", "bsc", &["utility"], vec![10.0, 90.0, 90.0, 5.0, 5.0, 5.0, 5.0, 5.0]),
        ];
        let c = cluster(&items, 0.85);
        assert!(c.iter().any(|g| g.token_ids.contains(&"alpha-inu".into()) && g.token_ids.contains(&"beta-cat".into())));
        assert!(c.iter().all(|g| g.evidence.iter().any(|e| e.contains("name is not used"))));
    }

    #[test]
    fn single_token_is_not_a_cluster() {
        assert!(cluster(&[tok("only", "ethereum", &["meme"], vec![1.0; 4])], 0.5).is_empty());
    }
}
