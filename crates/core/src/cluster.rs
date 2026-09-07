use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WalletCluster {
    pub cluster_id: String,
    pub members: Vec<String>,
    pub edge_count: u32,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Undirected transfer clusters. Hubs (CEX / burn) are skipped so they do not
/// collapse the graph. Correlation ≠ identity.
pub fn cluster(edges: &[(String, String)], skip: &HashSet<String>) -> Vec<WalletCluster> {
    let mut parent: HashMap<String, String> = HashMap::new();
    let mut kept = 0u32;

    fn find(parent: &mut HashMap<String, String>, x: &str) -> String {
        let p = parent.get(x).cloned().unwrap_or_else(|| x.to_string());
        if p != x {
            let root = find(parent, &p);
            parent.insert(x.to_string(), root.clone());
            root
        } else {
            parent.entry(x.to_string()).or_insert_with(|| x.to_string());
            p
        }
    }

    for (a, b) in edges {
        let a = a.to_lowercase();
        let b = b.to_lowercase();
        if a == b || skip.contains(&a) || skip.contains(&b) {
            continue;
        }
        kept += 1;
        let ra = find(&mut parent, &a);
        let rb = find(&mut parent, &b);
        if ra != rb {
            let (keep, drop) = if ra < rb { (ra, rb) } else { (rb, ra) };
            parent.insert(drop, keep);
        }
    }

    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    let nodes: Vec<String> = parent.keys().cloned().collect();
    for n in nodes {
        let r = find(&mut parent, &n);
        groups.entry(r).or_default().push(n);
    }

    let mut out: Vec<WalletCluster> = groups
        .into_iter()
        .filter(|(_, m)| m.len() >= 2)
        .map(|(id, mut members)| {
            members.sort();
            let n = members.len();
            WalletCluster {
                cluster_id: id,
                members,
                edge_count: kept,
                confidence: (0.28 + (n as f64) * 0.04).min(0.5),
                evidence: vec![
                    format!("{n} addresses linked by transfers in the indexed window"),
                    "shared path is not identity".into(),
                ],
            }
        })
        .collect();
    out.sort_by(|a, b| b.members.len().cmp(&a.members.len()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cex;

    #[test]
    fn cex_hub_does_not_merge_retail() {
        let binance = "0x28c6c06298d514db089934071355e5743bf21d60";
        let a = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let b = "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let skip = HashSet::from([binance.to_string()]);
        let edges = vec![
            (a.into(), binance.into()),
            (b.into(), binance.into()),
        ];
        assert!(cluster(&edges, &skip).is_empty());
        assert!(cex::is_hub(binance));
    }

    #[test]
    fn direct_transfer_forms_cluster() {
        let edges = vec![(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".into(),
        )];
        let c = cluster(&edges, &HashSet::new());
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].members.len(), 2);
        assert!(c[0].evidence.iter().any(|e| e.contains("not identity")));
    }
}
