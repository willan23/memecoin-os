use crate::ai;
use crate::alerts;
use crate::error::Result;
use crate::fixtures;
use crate::growth;
use crate::models::{
    Alert, DailyBriefing, DataState, TokenDefinition, TokenSnapshot, TokenSummary, ValidationStatus,
};
use crate::providers::{
    parse_github_repo, social_from_env, CoinGeckoProvider, DexScreenerProvider, GitHubDevProvider,
    LiquidityProvider, MarketProvider, OnchainProvider, PublicHolderProvider, SocialProvider,
};
use crate::quality::{
    apply_data_state, carry_development, carry_liquidity, carry_market, carry_onchain, carry_social,
    consensus_price, missing_development, missing_liquidity, missing_market, missing_onchain,
    missing_social, rollup_states,
};
use crate::risk;
use crate::scoring;
use chrono::Utc;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataMode {
    Live,
    Simulation,
}

impl DataMode {
    pub fn from_env() -> Self {
        match std::env::var("DATA_MODE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "simulation" | "simulated" | "fixture" => DataMode::Simulation,
            _ => DataMode::Live,
        }
    }

    pub fn is_simulation(self) -> bool {
        self == DataMode::Simulation
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeFlags {
    pub market: bool,
    pub dexscreener: bool,
    pub github: bool,
    pub holders: bool,
    pub social: bool,
}

impl RuntimeFlags {
    pub fn from_env() -> Self {
        Self {
            market: env_flag("FEATURE_COINGECKO", true),
            dexscreener: env_flag("FEATURE_DEXSCREENER", true),
            github: env_flag("FEATURE_GITHUB", true),
            holders: env_flag("FEATURE_HOLDERS", true),
            social: env_flag("FEATURE_SOCIAL", false),
        }
    }
}

fn env_flag(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(v) => matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"),
        Err(_) => default,
    }
}

pub struct IntelligenceEngine {
    market: Arc<dyn MarketProvider>,
    liquidity: Arc<dyn LiquidityProvider>,
    onchain: Arc<dyn OnchainProvider>,
    social: Arc<dyn SocialProvider>,
    github: GitHubDevProvider,
    mode: DataMode,
    flags: RuntimeFlags,
}

impl IntelligenceEngine {
    pub fn new(market: Arc<dyn MarketProvider>) -> Self {
        Self {
            market,
            liquidity: Arc::new(DexScreenerProvider::new()),
            onchain: Arc::new(PublicHolderProvider::new()),
            social: social_from_env(),
            github: GitHubDevProvider::new(),
            mode: DataMode::from_env(),
            flags: RuntimeFlags::from_env(),
        }
    }

    pub fn with_mode(mut self, mode: DataMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn live_default() -> Self {
        Self::new(Arc::new(CoinGeckoProvider::new()))
    }

    pub fn mode(&self) -> DataMode {
        self.mode
    }

    pub async fn snapshot(&self, def: &TokenDefinition) -> Result<TokenSnapshot> {
        self.snapshot_with_prior(def, None).await
    }

    pub async fn snapshot_with_prior(
        &self,
        def: &TokenDefinition,
        prior: Option<&TokenSnapshot>,
    ) -> Result<TokenSnapshot> {
        if self.mode.is_simulation() {
            return self.snapshot_simulated(def).await;
        }
        self.snapshot_live(def, prior).await
    }

    async fn snapshot_simulated(&self, def: &TokenDefinition) -> Result<TokenSnapshot> {
        let fx = fixtures::for_token(def);
        let live = if let Some(src) = &def.data_sources.market {
            if self.flags.market {
                self.market.fetch_market(&src.provider_id).await?
            } else {
                None
            }
        } else {
            None
        };
        let mut market = if let Some(m) = live {
            apply_data_state(m, false)
        } else {
            apply_data_state(fx.market, true)
        };
        let liquidity = apply_data_state(fx.liquidity, true);
        let onchain = apply_data_state(fx.onchain, true);
        let social = apply_data_state(fx.social, true);
        let development = apply_data_state(fx.development, true);
        market.data_state = if market.data_state == DataState::Missing {
            DataState::Simulated
        } else {
            market.data_state
        };
        self.assemble(
            def,
            market,
            liquidity,
            onchain,
            social,
            development,
            fixtures::timeline(&def.token_id),
            true,
        )
    }

    async fn snapshot_live(
        &self,
        def: &TokenDefinition,
        prior: Option<&TokenSnapshot>,
    ) -> Result<TokenSnapshot> {
        let mut cg = None;
        if self.flags.market {
            if let Some(src) = &def.data_sources.market {
                cg = self.market.fetch_market(&src.provider_id).await?;
            }
        }

        let mut dex_liq = None;
        let mut dex_price = None;
        if self.flags.dexscreener {
            if let Some(chain) = def.primary_chain() {
                if let Some((liq, px)) = self
                    .liquidity
                    .fetch_liquidity(&chain.id, &chain.contracts.token.address)
                    .await?
                {
                    dex_price = px;
                    dex_liq = Some(liq);
                }
            }
        }

        let mut market = cg.unwrap_or_else(missing_market);
        if let Some(px) = dex_price {
            if market.data_state.present() && market.price_usd > 0.0 && px > 0.0 {
                let (price, conf, status) =
                    consensus_price(&[(market.price_usd, market.provenance.confidence), (px, 0.75)]);
                market.price_usd = price;
                market.provenance.confidence = conf;
                market.provenance.validation_status = status.clone();
                if status == ValidationStatus::Conflict {
                    market.provenance.provider = "coingecko+dexscreener".into();
                    market.data_state = DataState::Conflict;
                }
            } else if !market.data_state.present() {
                market.price_usd = px;
                market.provenance.provider = "dexscreener".into();
                market.provenance.validation_status = ValidationStatus::Valid;
                market.provenance.confidence = 0.7;
                market.data_state = DataState::Live;
            }
        } else if market.data_state.present() {
            market = apply_data_state(market, false);
        }

        let liquidity = dex_liq
            .map(|l| apply_data_state(l, false))
            .unwrap_or_else(missing_liquidity);

        let mut onchain = missing_onchain();
        if self.flags.holders {
            if let Some(chain) = def.primary_chain() {
                if let Some(oc) = self
                    .onchain
                    .fetch_onchain(&chain.id, &chain.contracts.token.address)
                    .await?
                {
                    onchain = apply_data_state(oc, false);
                }
            }
        }
        onchain = carry_onchain(onchain, prior.map(|p| &p.onchain));

        let social = if self.flags.social {
            match self.social.fetch_social(&def.token_id).await? {
                Some(s) => apply_data_state(s, false),
                None => missing_social(),
            }
        } else {
            missing_social()
        };
        let social = carry_social(social, prior.map(|p| &p.social));

        let mut development = missing_development();
        if self.flags.github {
            if let Some(url) = def.socials.github.as_deref() {
                if let Some((owner, repo)) = parse_github_repo(url) {
                    if let Some(dev) = self.github.fetch_repo(&owner, &repo).await? {
                        development = apply_data_state(dev, false);
                    }
                }
            }
        }
        development = carry_development(development, prior.map(|p| &p.development));

        let market = carry_market(market, prior.map(|p| &p.market));
        let liquidity = carry_liquidity(liquidity, prior.map(|p| &p.liquidity));

        let timeline = vec![];
        self.assemble(
            def, market, liquidity, onchain, social, development, timeline, false,
        )
    }

    fn assemble(
        &self,
        def: &TokenDefinition,
        market: crate::models::MarketSnapshot,
        liquidity: crate::models::LiquiditySnapshot,
        onchain: crate::models::OnchainSnapshot,
        social: crate::models::SocialSnapshot,
        development: crate::models::DevelopmentSnapshot,
        timeline: Vec<crate::models::TimelineEvent>,
        simulated: bool,
    ) -> Result<TokenSnapshot> {
        let risk_report = risk::assess(def, &market, &liquidity, &onchain, &social, &development);
        let scores = scoring::ecosystem_health(
            &market,
            &liquidity,
            &onchain,
            &social,
            &development,
            &def.narratives,
            def.features.governance,
            def.features.treasury,
            risk_report.score,
        );
        let genome = scoring::genome(
            &market,
            &liquidity,
            &onchain,
            &social,
            &development,
            &def.narratives,
            def.features.governance,
            def.features.treasury,
            risk_report.score,
        );
        let growth_ops =
            growth::opportunities(def, &scores, &genome, &social, &liquidity, &development);

        let (price_series, health_series) = if simulated {
            (
                fixtures::series(
                    &format!("{}-price", def.token_id),
                    48,
                    market.price_usd.max(1e-18),
                    0.04,
                ),
                fixtures::series(
                    &format!("{}-health", def.token_id),
                    24,
                    scores.value,
                    0.015,
                ),
            )
        } else {
            (vec![], vec![])
        };

        let summary = TokenSummary {
            id: def.token_id.clone(),
            symbol: def.symbol.clone(),
            name: def.name.clone(),
            status: def.status.clone(),
            verification: def.verification.clone(),
            narratives: def.narratives.clone(),
            primary_chain: def
                .primary_chain()
                .map(|c| c.id.clone())
                .unwrap_or_else(|| "unknown".into()),
            website: def.website.clone(),
            description: def.description.clone(),
            contract: def
                .primary_chain()
                .map(|c| c.contracts.token.address.clone()),
            chains: def.chains.iter().map(|c| c.id.clone()).collect(),
        };

        let data_state = rollup_states(&[
            market.data_state.clone(),
            liquidity.data_state.clone(),
            onchain.data_state.clone(),
            social.data_state.clone(),
            development.data_state.clone(),
        ]);

        let mut snap = TokenSnapshot {
            token: summary,
            market,
            liquidity,
            onchain,
            social,
            development,
            scores,
            genome,
            risk: risk_report,
            growth: growth_ops,
            timeline,
            briefing: DailyBriefing {
                token_id: def.token_id.clone(),
                as_of: Utc::now(),
                ecosystem_health: 0.0,
                headline: String::new(),
                changes: vec![],
                risks: vec![],
                opportunities: vec![],
                investigation: String::new(),
                confidence: 0.0,
                evidence: vec![],
            },
            observations: vec![],
            price_series,
            health_series,
            as_of: Utc::now(),
            data_state,
        };
        snap.observations = ai::observations(&snap);
        snap.briefing = ai::briefing(&snap);
        Ok(snap)
    }

    pub fn alerts_from(&self, snaps: &[TokenSnapshot]) -> Vec<Alert> {
        let mut alerts = Vec::new();
        for s in snaps {
            alerts.extend(alerts::evaluate(s, None));
        }
        alerts.sort_by(|a, b| b.fired_at.cmp(&a.fired_at));
        alerts
    }

    pub fn alerts_with_prev(
        &self,
        snap: &TokenSnapshot,
        prev: Option<&TokenSnapshot>,
    ) -> Vec<Alert> {
        alerts::evaluate(snap, prev)
    }
}
