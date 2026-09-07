use crate::models::{
    DevelopmentSnapshot, EcosystemScore, Genome, LiquiditySnapshot, Opportunity, SocialSnapshot,
    TokenDefinition,
};

pub fn opportunities(
    def: &TokenDefinition,
    score: &EcosystemScore,
    genome: &Genome,
    social: &SocialSnapshot,
    liq: &LiquiditySnapshot,
    dev: &DevelopmentSnapshot,
) -> Vec<Opportunity> {
    let mut out = Vec::new();
    let util = dim(genome, "utility");
    let development = dim(genome, "development");
    let social_m = dim(genome, "social_momentum");
    let gov = dim(genome, "governance");
    let treasury = dim(genome, "treasury");

    if social.data_state.present() && social.organicness < 0.7 && social.bot_probability > 0.2 {
        out.push(Opportunity {
            id: "ai-utility-layer".into(),
            title: "AI utility layer the narrative already implies".into(),
            impact: 86.0,
            cost: "medium".into(),
            risk: "low".into(),
            confidence: 0.78,
            evidence: vec![
                format!("utility genome {util:.0}"),
                "narrative includes ai".into(),
                format!("community score component present in health {:.0}", score.value),
            ],
            expected_metric: "utility score / retained holders".into(),
        });
    }

    if development < 30.0 && dev.data_state.present() {
        out.push(Opportunity {
            id: "public-dev-cadence".into(),
            title: "Publish a visible development cadence".into(),
            impact: 74.0,
            cost: "low".into(),
            risk: "low".into(),
            confidence: 0.82,
            evidence: vec![
                format!("commits_30d={}", dev.commits_30d),
                format!("last_commit_days={:?}", dev.last_commit_days),
            ],
            expected_metric: "developer activity / transparency".into(),
        });
    }

    if social.data_state.present() && social.organicness < 0.7 && social.bot_probability > 0.2 {
        out.push(Opportunity {
            id: "organic-moderation".into(),
            title: "Raise organicness via moderation, not volume".into(),
            impact: 68.0,
            cost: "low".into(),
            risk: "low".into(),
            confidence: 0.73,
            evidence: vec![
                format!("organicness={:.2}", social.organicness),
                format!("bot_probability={:.2}", social.bot_probability),
                format!("spam_ratio={:.2}", social.spam_ratio),
            ],
            expected_metric: "social momentum quality".into(),
        });
    }

    if liq.data_state.present() && (liq.spread_bps > 40.0 || liq.lp_change_7d_pct < 0.0) {
        out.push(Opportunity {
            id: "mm-depth".into(),
            title: "Improve DEX depth and spread (organic LP, not wash volume)".into(),
            impact: 80.0,
            cost: "high".into(),
            risk: "medium".into(),
            confidence: 0.7,
            evidence: vec![
                format!("spread_bps={:.0}", liq.spread_bps),
                format!("liquidity_usd={:.0}", liq.liquidity_usd),
                format!("lp_change_7d={:.1}%", liq.lp_change_7d_pct),
            ],
            expected_metric: "liquidity score / market health".into(),
        });
    }

    if gov < 20.0 {
        out.push(Opportunity {
            id: "governance-surface".into(),
            title: "Expose a public decision log even without a DAO".into(),
            impact: 55.0,
            cost: "low".into(),
            risk: "low".into(),
            confidence: 0.66,
            evidence: vec!["governance genome below 20".into()],
            expected_metric: "transparency / governance score".into(),
        });
    }

    if treasury < 20.0 && def.features.treasury {
        out.push(Opportunity {
            id: "treasury-reporting".into(),
            title: "Publish treasury balances and runway".into(),
            impact: 70.0,
            cost: "low".into(),
            risk: "low".into(),
            confidence: 0.71,
            evidence: vec!["treasury feature enabled but score remains low".into()],
            expected_metric: "treasury health".into(),
        });
    }

    if social_m < 40.0 && social.data_state.present() {
        out.push(Opportunity {
            id: "education-not-hype".into(),
            title: "Education and documentation quests instead of hype cycles".into(),
            impact: 60.0,
            cost: "medium".into(),
            risk: "low".into(),
            confidence: 0.69,
            evidence: vec![format!("social momentum {social_m:.0}")],
            expected_metric: "unique accounts / organicness".into(),
        });
    }

    for o in &mut out {
        if !o.evidence.iter().any(|e| e.contains("buy/sell")) {
            o.evidence.push("Not a buy/sell recommendation.".into());
        }
    }
    out.sort_by(|a, b| b.impact.partial_cmp(&a.impact).unwrap());
    out.into_iter().take(5).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{EcosystemScore, Genome, GenomeDimension};
    use crate::quality::{missing_development, missing_liquidity, missing_social};

    #[test]
    fn missing_social_does_not_fire_organic_or_buy() {
        let def = crate::onboard_definition(crate::OnboardRequest {
            name: "X".into(),
            symbol: "X".into(),
            chain: "ethereum".into(),
            address: "0x6982508145454Ce325dDbE47a25d4ec3d2311933".into(),
            website: None,
            decimals: None,
        })
        .unwrap();
        let score = EcosystemScore {
            algorithm: "t".into(),
            algorithm_version: "1".into(),
            value: 40.0,
            confidence: 0.4,
            as_of: chrono::Utc::now(),
            components: vec![],
            why: "t".into(),
        };
        let genome = Genome {
            algorithm_version: "1".into(),
            dimensions: vec![GenomeDimension {
                id: "social_momentum".into(),
                label: "S".into(),
                value: 10.0,
            }],
        };
        let ops = opportunities(
            &def,
            &score,
            &genome,
            &missing_social(),
            &missing_liquidity(),
            &missing_development(),
        );
        assert!(!ops.iter().any(|o| o.id == "organic-moderation" || o.id == "mm-depth"));
        assert!(ops.iter().all(|o| o.evidence.iter().any(|e| e.contains("buy/sell"))));
    }
}

fn dim(g: &Genome, id: &str) -> f64 {
    g.dimensions
        .iter()
        .find(|d| d.id == id)
        .map(|d| d.value)
        .unwrap_or(0.0)
}
