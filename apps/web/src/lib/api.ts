import { operatorHeaders } from "./operator";
import type { AgentAnswer, Overview, TokenSnapshot } from "./types";

export type TwinField = {
  data_state: string;
  confidence: number;
  summary: string;
  evidence: string[];
};

export type TwinState = {
  timestamp: string;
  ecosystem_id: string;
  token_id: string;
  market_state: TwinField;
  liquidity_state: TwinField;
  holder_state: TwinField;
  wallet_state: TwinField;
  whale_state: TwinField;
  developer_state: TwinField;
  social_state: TwinField;
  narrative_state: TwinField;
  risk_state: TwinField;
  governance_state: TwinField;
  network_state: TwinField;
  confidence: number;
  freshness: string;
  note: string;
};

export type EvidenceClaimRow = {
  id: string;
  claim: string;
  relation: "supports" | "contradicts" | "insufficient" | string;
  evidence: string[];
  data_state: string;
  confidence: number;
  stored_at?: string | null;
};

async function get<T>(path: string): Promise<T> {
  const res = await fetch(path, { cache: "no-store" });
  if (!res.ok) throw new Error(`API ${res.status} ${path}`);
  return res.json();
}

/** Same-origin on Firebase Hosting; local SSE hits the API directly. */
export function eventsUrl() {
  const api = process.env.NEXT_PUBLIC_API_URL;
  if (!api || api === "same-origin") {
    if (typeof window !== "undefined" && window.location.hostname.endsWith("web.app")) {
      return "/v2/events";
    }
    if (typeof window !== "undefined" && window.location.hostname.endsWith("firebaseapp.com")) {
      return "/v2/events";
    }
    return "http://127.0.0.1:8080/v2/events";
  }
  return `${api.replace(/\/$/, "")}/v2/events`;
}

export const api = {
  community: () =>
    get<{
      tokens: {
        id: string;
        symbol: string;
        name: string;
        website?: string | null;
        socials: {
          twitter?: string | null;
          telegram?: string | null;
          github?: string | null;
          discord?: string | null;
        };
        social_state: string;
        mentions_24h?: number | string | null;
        chat_state?: string;
        chat_messages?: number;
        chat_messages_24h?: number;
        note?: string;
      }[];
      chat?: { live?: boolean; messages?: number; messages_24h?: number };
      note?: string;
    }>("/v1/community"),
  connectStatus: () =>
    get<{
      discord: { configured: boolean; start: string; note: string };
      telegram: { configured: boolean; start: string; bot_username?: string | null; note: string };
      disclaimer?: string;
    }>("/v1/connect"),
  telegramStart: (tokenId?: string) =>
    get<{ url: string; note?: string }>(
      tokenId ? `/v1/connect/telegram?token_id=${encodeURIComponent(tokenId)}` : "/v1/connect/telegram",
    ),
  communityRooms: () =>
    get<{
      live?: boolean;
      messages?: number;
      messages_24h?: number;
      rooms: { id: string; label: string; messages?: number; messages_24h?: number }[];
      disclaimer?: string;
    }>("/v1/community/rooms"),
  communityMessages: (room: string) =>
    get<{
      room_id: string;
      messages: { id: string; handle: string; body: string; created_at: string }[];
    }>(`/v1/community/rooms/${encodeURIComponent(room)}/messages`),
  communitySession: async (handle: string) => {
    const res = await fetch("/v1/community/session", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ handle }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json() as Promise<{ session_id: string; handle: string }>;
  },
  communityPost: async (room: string, body: string, sessionId: string) => {
    const res = await fetch(`/v1/community/rooms/${encodeURIComponent(room)}/messages`, {
      method: "POST",
      headers: { "content-type": "application/json", "x-community-session": sessionId },
      body: JSON.stringify({ body }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json() as Promise<{
      id: string;
      handle: string;
      body: string;
      created_at: string;
      session_id?: string;
    }>;
  },
  overview: () => get<Overview>("/v1/overview"),
  token: (id: string) => get<TokenSnapshot>(`/v1/tokens/${id}`),
  compare: (ids: string[]) =>
    get<{ tokens: TokenSnapshot[] }>(`/v1/compare?ids=${ids.join(",")}`),
  rankings: () =>
    get<{
      health: { id: string; symbol: string; value: number }[];
      momentum: { id: string; symbol: string; value: number }[];
      risk: { id: string; symbol: string; value: number; level: string }[];
      development: { id: string; symbol: string; value: number }[];
      community: { id: string; symbol: string; value: number }[];
      liquidity: { id: string; symbol: string; value: number }[];
    }>("/v1/rankings"),
  alerts: () =>
    get<{
      alerts: {
        id: string;
        token_id: string;
        kind: string;
        severity: string;
        title: string;
        body: string;
        evidence: string[];
        fired_at: string;
      }[];
    }>("/v1/alerts"),
  research: async (question: string, tokenId?: string) => {
    const res = await fetch("/v1/research", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ question, token_id: tokenId }),
    });
    if (!res.ok) throw new Error(`API ${res.status} /v1/research`);
    return res.json() as Promise<{
      executive_summary: string;
      markdown: string;
      confidence: number;
      model_id: string;
      grounded: boolean;
      policy: { execute: boolean };
      unknown: string[];
      limitations: string[];
      sections: { heading: string; body: string; data_state: string }[];
    }>;
  },
  ask: async (question: string, tokenId?: string): Promise<AgentAnswer> => {
    const res = await fetch("/v1/ai/query", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ question, token_id: tokenId }),
    });
    if (!res.ok) throw new Error(`API ${res.status}`);
    return res.json();
  },
  onboard: async (body: {
    name: string;
    symbol: string;
    chain: string;
    address: string;
    website?: string;
    decimals?: number;
  }): Promise<TokenSnapshot> => {
    const res = await fetch("/v1/tokens", {
      method: "POST",
      headers: { "content-type": "application/json", ...operatorHeaders() },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  baselines: (id: string) =>
    get<{
      token_id: string;
      baselines: {
        metric: string;
        window: string;
        n: number;
        mean: number;
        stddev: number;
        last: number;
        z_score: number | null;
        mad?: number;
        percentile_25?: number | null;
        percentile_75?: number | null;
        data_state: string;
        evidence: string[];
      }[];
    }>(`/v1/tokens/${id}/baselines`),
  claims: async (id: string) => {
    const raw = await get<{
      data?: {
        claims: EvidenceClaimRow[];
        persisted?: EvidenceClaimRow[];
        persisted_count?: number;
        note?: string;
      };
      claims?: EvidenceClaimRow[];
      persisted?: EvidenceClaimRow[];
      persisted_count?: number;
      note?: string;
    }>(`/v1/ecosystems/${id}/claims`);
    const inner = raw.data ?? raw;
    return {
      claims: inner.claims ?? [],
      persisted: inner.persisted ?? [],
      persisted_count: inner.persisted_count ?? (inner.persisted ?? []).length,
      note: inner.note,
    };
  },
  pools: (id: string) =>
    get<{
      token_id: string;
      pools: { chain_id: string; pair_address: string; dex?: string | null; liquidity_usd?: number | null }[];
    }>(`/v1/tokens/${id}/pools`),
  wallets: (id: string) =>
    get<{
      wallets: {
        address: string;
        classification: string;
        confidence: number;
        share_pct?: number | null;
        evidence: string[];
      }[];
      disclaimer?: string;
    }>(`/v1/tokens/${id}/wallets`),
  discovery: () =>
    get<{ candidates: { chain_id: string; address: string; symbol?: string | null; name?: string | null; status: string; score: number; auto_verified: boolean; already_tracked?: boolean; liquidity_usd?: number | null; evidence?: string[] }[]; disclaimer?: string }>(
      "/v1/discovery",
    ),
  narratives: () =>
    get<{ narratives: { id: string; label: string; state: string; confidence: number; token_ids: string[]; mention_velocity: string; unique_accounts: string; evidence?: string[] }[] }>(
      "/v1/narratives",
    ),
  genomeClusters: () =>
    get<{ clusters: { cluster_id: string; label: string; token_ids: string[]; confidence: number }[] }>("/v1/genome-clusters"),
  verification: (id: string) =>
    get<{ token_id: string; level: string; declared_level?: string; reasons?: string[]; evidence?: string[]; disclaimer?: string; note?: string }>(
      `/v1/tokens/${id}/verification`,
    ),
  transfers: (id: string) =>
    get<{ transfers: { tx_hash: string; from: string; to: string; amount_raw: string }[]; note?: string }>(
      `/v1/tokens/${id}/transfers`,
    ),
  whaleEvents: (id: string) =>
    get<{ events: { direction: string; wallet?: string; exchange?: string; amount_usd?: number | null }[] }>(
      `/v1/tokens/${id}/whale-events`,
    ),
  clusters: (id: string) =>
    get<{ clusters: { cluster_id: string; members: string[]; confidence: number }[] }>(`/v1/tokens/${id}/clusters`),
  history: (id: string, range = "30d") =>
    get<{
      token_id: string;
      range_days: number;
      points: {
        t: string;
        price_usd: number | null;
        market_cap_usd: number | null;
        volume_24h_usd: number | null;
        liquidity_usd: number | null;
        health: number | null;
        risk: number | null;
        data_state: string;
      }[];
    }>(`/v1/tokens/${id}/history?range=${range}`),
  webhooks: () =>
    get<{
      webhooks: {
        id: string;
        kind: string;
        url: string;
        chat_id?: string | null;
        token_id?: string | null;
        enabled: boolean;
        digest_daily: boolean;
      }[];
      degraded?: boolean;
    }>("/v1/webhooks"),
  createWebhook: async (body: {
    kind: string;
    url: string;
    chat_id?: string;
    token_id?: string;
    digest_daily?: boolean;
  }) => {
    const res = await fetch("/v1/webhooks", {
      method: "POST",
      headers: { "content-type": "application/json", ...operatorHeaders() },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  deleteWebhook: async (id: string) => {
    const res = await fetch(`/v1/webhooks/${id}`, { method: "DELETE" });
    if (!res.ok && res.status !== 204) throw new Error(await res.text());
  },
  flags: () =>
    get<{ key: string; enabled: boolean; reason?: string | null }[] | { flags?: { key: string; enabled: boolean; reason?: string | null }[] }>("/v1/flags"),
  setFlag: async (key: string, enabled: boolean) => {
    const res = await fetch(`/v1/flags/${encodeURIComponent(key)}`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ enabled }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  keys: () =>
    get<{
      keys: { id: string; name: string; prefix: string; role: string; revoked: boolean }[];
      auth_required?: boolean;
      degraded?: boolean;
    }>("/v1/keys"),
  createKey: async (name: string, role = "api_client") => {
    const res = await fetch("/v1/keys", {
      method: "POST",
      headers: { "content-type": "application/json", ...operatorHeaders() },
      body: JSON.stringify({ name, role }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json() as Promise<{ secret: string; prefix: string; id: string; note: string }>;
  },
  revokeKey: async (id: string) => {
    const res = await fetch(`/v1/keys/${id}`, { method: "DELETE" });
    if (!res.ok && res.status !== 204) throw new Error(await res.text());
  },
  alertRules: () =>
    get<{ rules: { kind: string; enabled: boolean; reason?: string | null }[] }>("/v1/alerts/rules"),
  setAlertRule: async (kind: string, enabled: boolean) => {
    const res = await fetch("/v1/alerts/rules", {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ kind, enabled }),
    });
    if (!res.ok) throw new Error(await res.text());
  },
  ackAlert: async (id: string) => {
    const res = await fetch(`/v1/alerts/${id}/ack`, { method: "POST" });
    if (!res.ok && res.status !== 204) throw new Error(await res.text());
  },
  refreshToken: async (id: string): Promise<TokenSnapshot> => {
    const res = await fetch(`/v1/tokens/${id}/refresh`, { method: "POST" });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  refreshAll: async () => {
    const res = await fetch("/v1/jobs/refresh", { method: "POST" });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  status: () => get<Record<string, unknown>>("/health"),
  ecosystems: () =>
    get<{
      ecosystems: {
        id: string;
        name: string;
        lifecycle_phase: string;
        discovery_score: number;
        intelligence_score: number;
        risk_score: number;
        confidence: number;
        verification_status: string;
        primary_chain: string;
        auto_verified: boolean;
        data_freshness: string;
      }[];
      note?: string;
    }>("/v2/ecosystems"),
  ecosystem: (id: string) => get<Record<string, unknown>>(`/v2/ecosystems/${id}`),
  twin: (id: string) => get<{ twin: TwinState; mode?: string }>(`/v2/ecosystems/${id}/twin`),
  twinState: (id: string, ago?: string) =>
    get<{ twin: TwinState; mode?: string }>(
      `/v2/ecosystems/${id}/twin/state${ago ? `?ago=${ago}` : ""}`,
    ),
  twinGraph: (id: string) =>
    get<{
      ecosystem_id: string;
      entities: { id: string; kind: string; label: string; importance: number; x: number; y: number }[];
      relationships: { source: string; target: string; relationship_type: string; confidence: number }[];
      note?: string;
    }>(`/v2/ecosystems/${id}/twin/graph`),
  twinEvolution: (id: string) =>
    get<{
      what_changed: { field: string; before: string; after: string; why: string; confidence: number }[];
      note?: string;
    }>(`/v2/ecosystems/${id}/twin/evolution`),
  twinEvidence: (id: string) =>
    get<{
      claims: { claim: string; confidence: number; evidence: string[]; counter_evidence: string[]; freshness: string }[];
      investigate_next: { title: string; reason: string; priority: number; evidence: string[]; confidence: number }[];
    }>(`/v2/ecosystems/${id}/evidence`),
  twinSimilar: (id: string) =>
    get<{ similar: { a: string; b: string; similarity: number; disclaimer: string }[] }>(`/v2/ecosystems/${id}/similar`),
  twinLifecycle: (id: string) =>
    get<{ phase: string; evidence?: string[]; note?: string }>(`/v2/ecosystems/${id}/lifecycle`),
  simulate: async (body: {
    ecosystem_id: string;
    liquidity_pct?: number;
    holder_growth_pct?: number;
    whale_selling_pct?: number;
    developer_pct?: number;
  }) => {
    const res = await fetch("/v2/simulations", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`API ${res.status} /v2/simulations`);
    return res.json() as Promise<{
      id?: string | null;
      result: {
        banner: string;
        current_health: number;
        projected_health: number;
        current_risk: number;
        projected_risk: number;
        delta_health: number;
        delta_risk: number;
        confidence: number;
        notes: string[];
        sensitivity: { variable: string; impact: string }[];
        projected_twin_note: string;
      };
    }>;
  },
  copilot: async (question: string, ecosystemId?: string) => {
    const res = await fetch("/v2/copilot", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ question, ecosystem_id: ecosystemId }),
    });
    if (!res.ok) throw new Error(`API ${res.status} /v2/copilot`);
    return res.json() as Promise<{
      research?: { executive_summary?: string; markdown?: string };
      investigate_next?: { title: string; reason: string }[];
      execute?: boolean;
      note?: string;
    }>;
  },
  twinReplay: (id: string, range = "24h") =>
    get<{
      frames: { t: string; kind: string; title: string; source: string; evidence: string[] }[];
      note?: string;
    }>(`/v2/ecosystems/${id}/twin/replay?range=${range}`),
  whaleBehaviour: (id: string) =>
    get<{
      behaviour: {
        pattern: string;
        in_events: number;
        out_events: number;
        cex_events: number;
        confidence: number;
        note: string;
        evidence: string[];
      };
    }>(`/v2/ecosystems/${id}/whales`),
  compareTwins: (a: string, b: string) =>
    get<{
      similarity: { similarity: number; disclaimer: string; similarities: string[]; differences: string[] };
      a: { ecosystem: { name: string; lifecycle_phase: string } };
      b: { ecosystem: { name: string; lifecycle_phase: string } };
    }>(`/v2/ecosystems/compare?ids=${a},${b}`),
  briefing: () =>
    get<{
      items: {
        ecosystem_id: string;
        name: string;
        lifecycle: string;
        headline: string;
        changes: string[];
        risks: string[];
        health: number;
        confidence: number;
        data_state: string;
      }[];
      note?: string;
    }>("/v2/briefing"),
  billing: () =>
    get<{
      tenant_id?: string;
      plan?: string;
      metering_enforce?: boolean;
      limits?: {
        api_requests_day: number;
        ai_requests_day: number;
        research_day: number;
        tokens_monitored: number;
      };
      usage_today?: Record<string, number>;
      stripe?: {
        processor?: string;
        configured?: boolean;
        keys_missing?: boolean;
        state?: string;
        checkout?: string;
        webhook?: string;
        prices_mapped?: {
          pro?: boolean;
          research?: boolean;
          growth?: boolean;
          enterprise?: boolean;
          api?: boolean;
        };
        note?: string;
      };
      note?: string;
      degraded?: boolean;
    }>("/v1/billing"),
  branding: () =>
    get<{
      tenant_id?: string;
      branding: {
        display_name?: string | null;
        logo_url?: string | null;
        accent?: string | null;
        custom_domain?: string | null;
      };
      note?: string;
    }>("/v1/branding"),
  saveBranding: async (
    tenantId: string,
    body: { display_name?: string; logo_url?: string; accent?: string; custom_domain?: string }
  ) => {
    const res = await fetch(`/v1/tenants/${tenantId}/branding`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`API ${res.status} branding`);
    return res.json();
  },
  watchlist: () => get<{ token_ids: string[]; note?: string }>("/v1/watchlist"),
  saveWatchlist: async (tokenIds: string[]) => {
    const res = await fetch("/v1/watchlist", {
      method: "PUT",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ token_ids: tokenIds }),
    });
    if (!res.ok) throw new Error(`API ${res.status} watchlist`);
    return res.json() as Promise<{ token_ids: string[] }>;
  },
  queue: () =>
    get<{ pending?: number; locked?: number; degraded?: boolean; note?: string }>("/v1/queue"),
  sso: () =>
    get<{ configured: boolean; issuer?: string; note?: string; redirect_uri?: string; scopes?: string; provider_hint?: string }>(
      "/v1/auth/sso"
    ),
  tenants: () =>
    get<{
      tenants: {
        id: string;
        slug: string;
        name: string;
        plan: string;
        status: string;
        branding?: { display_name?: string; logo_url?: string; accent?: string; custom_domain?: string };
      }[];
      degraded?: boolean;
    }>("/v1/tenants"),
  storage: () =>
    get<{ path: string; blobs: number; raw_store_enabled: boolean; note?: string }>("/v1/admin/storage"),
  projects: () =>
    get<{
      projects: {
        id: string;
        symbol: string;
        name: string;
        claim_status: string;
        verification: string;
        health: number;
        risk: number;
        data_state: string;
      }[];
      disclaimer?: string;
    }>("/v1/projects"),
  project: (id: string) =>
    get<{
      token_id: string;
      symbol: string;
      name: string;
      claim: {
        status: string;
        label: string;
        claimant_label?: string | null;
        claimed_at?: string | null;
      };
      official: {
        website: string;
        twitter: string;
        telegram: string;
        discord: string;
        github: string;
        docs: string;
        roadmap: string;
      };
      verification: { declared: string; computed: string; disclaimer?: string };
      health: number;
      risk: number;
      data_state: string;
      disclaimer?: string;
    }>(`/v1/projects/${id}`),
  claimProject: async (id: string, claimant_label?: string) => {
    const res = await fetch(`/v1/projects/${id}/claim`, {
      method: "POST",
      headers: { "content-type": "application/json", ...operatorHeaders() },
      body: JSON.stringify({ claimant_label }),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  unclaimProject: async (id: string) => {
    const res = await fetch(`/v1/projects/${id}/unclaim`, {
      method: "POST",
      headers: { "content-type": "application/json", ...operatorHeaders() },
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
  projectAlerts: (id: string) =>
    get<{
      token_id: string;
      alerts: {
        id: string;
        token_id: string;
        kind: string;
        severity: string;
        title: string;
        body: string;
        evidence: string[];
        fired_at: string;
      }[];
      catalog: { kind: string; label: string; availability: string; reason: string }[];
      webhooks?: { id: string; kind: string; url: string; token_id?: string | null }[];
      disclaimer?: string;
    }>(`/v1/projects/${id}/alerts`),
  patchProjectProfile: async (
    id: string,
    body: {
      website?: string;
      twitter?: string;
      telegram?: string;
      discord?: string;
      github?: string;
      docs?: string;
      roadmap?: string;
    },
  ) => {
    const res = await fetch(`/v1/projects/${id}/profile`, {
      method: "PATCH",
      headers: { "content-type": "application/json", ...operatorHeaders() },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  },
};
