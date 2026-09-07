export type Provenance = {
  source: string;
  provider: string;
  timestamp: string;
  freshness_secs: number;
  confidence: number;
  validation_status: string;
  raw_reference: string | null;
};

export type TokenSummary = {
  id: string;
  symbol: string;
  name: string;
  status: string;
  verification: string;
  narratives: string[];
  primary_chain: string;
  website?: string | null;
  description?: string | null;
  contract?: string | null;
  chains: string[];
};

export type ScoreDriver = { label: string; delta: number; direction: string };
export type ScoreComponent = {
  id: string;
  label: string;
  value: number;
  weight: number;
  drivers: ScoreDriver[];
};

export type Opportunity = {
  id: string;
  title: string;
  impact: number;
  cost: string;
  risk: string;
  confidence: number;
  evidence: string[];
  expected_metric: string;
};

export type ObservationCard = {
  observation: string;
  evidence: string[];
  confidence: number;
  interpretation: string;
  risk?: string | null;
};

export type TimelineEvent = {
  at: string;
  kind: string;
  title: string;
  delta?: string | null;
  confidence: number;
  source: string;
};

export type TokenSnapshot = {
  token: TokenSummary;
  market: {
    price_usd: number;
    market_cap_usd: number;
    fdv_usd: number;
    volume_24h_usd: number;
    change_24h_pct: number;
    ath_usd?: number | null;
    ath_distance_pct?: number | null;
    provenance: Provenance;
    data_state?: string;
  };
  liquidity: {
    liquidity_usd: number;
    pool_count: number;
    spread_bps: number;
    depth_plus_2pct_usd: number;
    depth_minus_2pct_usd: number;
    lp_change_7d_pct: number;
    buy_sell_imbalance: number;
    provenance: Provenance;
    data_state?: string;
  };
  onchain: {
    holders: number;
    active_holders_30d: number;
    new_holders_7d: number;
    retained_holders_30d_pct: number;
    lost_holders_7d: number;
    whale_holders: number;
    top10_concentration_pct: number;
    top50_concentration_pct: number;
    transfers_24h: number;
    unique_senders_24h: number;
    large_transfers_24h: number;
    exchange_inflow_usd: number;
    exchange_outflow_usd: number;
    provenance: Provenance;
    data_state?: string;
  };
  social: {
    mentions_24h: number;
    unique_accounts_24h: number;
    engagement_score: number;
    sentiment_net: number;
    bot_probability: number;
    organicness: number;
    narrative_diversity: number;
    dominant_topics: string[];
    spam_ratio: number;
    provenance: Provenance;
    data_state?: string;
  };
  development: {
    commits_30d: number;
    active_contributors_30d: number;
    releases_90d: number;
    last_commit_days?: number | null;
    open_issues: number;
    activity_score: number;
    provenance: Provenance;
    data_state?: string;
  };
  scores: {
    algorithm: string;
    algorithm_version: string;
    value: number;
    confidence: number;
    as_of: string;
    components: ScoreComponent[];
    why: string;
  };
  genome: {
    algorithm_version: string;
    dimensions: { id: string; label: string; value: number }[];
  };
  risk: {
    algorithm: string;
    algorithm_version: string;
    score: number;
    level: string;
    confidence: number;
    as_of: string;
    factors: {
      id: string;
      label: string;
      score: number;
      level: string;
      evidence: string[];
    }[];
    why: string;
  };
  growth: Opportunity[];
  timeline: TimelineEvent[];
  briefing: {
    token_id: string;
    as_of: string;
    ecosystem_health: number;
    headline: string;
    changes: string[];
    risks: string[];
    opportunities: string[];
    investigation: string;
    confidence: number;
  };
  observations: ObservationCard[];
  price_series: { t: string; v: number }[];
  health_series: { t: string; v: number }[];
  as_of: string;
  data_state?: string;
};

export type TokenCard = {
  token: TokenSummary;
  health: number;
  health_confidence: number;
  risk: number;
  risk_level: string;
  price_usd?: number;
  market_cap_usd: number;
  volume_24h_usd: number;
  change_24h_pct: number;
  liquidity_usd: number;
  holders: number;
  data_state?: string;
  market_state?: string;
  holders_state?: string;
};

export type Overview = {
  tracked_tokens: number;
  total_market_cap_usd: number;
  global_volume_24h_usd: number;
  active_ecosystems: number;
  top_momentum: TokenSummary[];
  highest_health: { id: string; symbol: string; value: number }[];
  highest_risk: { id: string; symbol: string; value: number; level: string }[];
  tokens: TokenCard[];
  disclaimer: string;
};

export type AgentAnswer = {
  token_id?: string | null;
  question: string;
  answer: string;
  observations: ObservationCard[];
  confidence: number;
  policy: Record<string, boolean>;
  model: string;
  grounded: boolean;
};
