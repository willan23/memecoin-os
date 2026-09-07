export type MemeCoinOsClientOptions = {
  baseUrl?: string;
};

export class MemeCoinOsClient {
  constructor(private readonly options: MemeCoinOsClientOptions = {}) {}

  private url(path: string) {
    const base = this.options.baseUrl ?? "http://127.0.0.1:8080";
    return `${base.replace(/\/$/, "")}${path}`;
  }

  async overview() {
    return this.get("/v1/overview");
  }

  async tokens() {
    return this.get("/v1/tokens");
  }

  async token(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}`);
  }

  async compare(ids: string[]) {
    return this.get(`/v1/compare?ids=${ids.map(encodeURIComponent).join(",")}`);
  }

  async rankings() {
    return this.get("/v1/rankings");
  }

  async alerts() {
    return this.get("/v1/alerts");
  }

  async health() {
    return this.get("/health");
  }

  async history(id: string, range = "30d") {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/history?range=${encodeURIComponent(range)}`);
  }

  async webhooks() {
    return this.get("/v1/webhooks");
  }

  async flags() {
    return this.get("/v1/flags");
  }

  async keys() {
    return this.get("/v1/keys");
  }

  async historyCsv(id: string, range = "30d") {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/history.csv?range=${encodeURIComponent(range)}`);
  }

  async baselines(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/baselines`);
  }

  async pools(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/pools`);
  }

  async wallets(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/wallets`);
  }

  async indexer() {
    return this.get("/v1/indexer");
  }

  async transfers(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/transfers`);
  }

  async whaleEvents(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/whale-events`);
  }

  async clusters(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/clusters`);
  }

  async discovery() {
    return this.get("/v1/discovery");
  }

  async narratives() {
    return this.get("/v1/narratives");
  }

  async genomeClusters() {
    return this.get("/v1/genome-clusters");
  }

  async verification(id: string) {
    return this.get(`/v1/tokens/${encodeURIComponent(id)}/verification`);
  }

  async research(question: string, tokenId?: string) {
    const res = await fetch(this.url("/v1/research"), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ question, token_id: tokenId }),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async refreshToken(id: string) {
    const res = await fetch(this.url(`/v1/tokens/${encodeURIComponent(id)}/refresh`), {
      method: "POST",
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async billing() {
    return this.get("/v1/billing");
  }

  async queue() {
    return this.get("/v1/queue");
  }

  async tenants() {
    return this.get("/v1/tenants");
  }

  async sso() {
    return this.get("/v1/auth/sso");
  }

  async discoveryV2() {
    return this.get("/v2/discovery");
  }

  async ecosystems() {
    return this.get("/v2/ecosystems");
  }

  async exchangeHealth() {
    return this.get("/v1/health");
  }

  async exchangeEcosystems() {
    return this.get("/v1/ecosystems");
  }

  async exchangeEcosystem(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}`);
  }

  async exchangeAsset(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/asset`);
  }

  async exchangeTwin(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/twin`);
  }

  async exchangeRisk(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/risk`);
  }

  async exchangeNarratives(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/narratives`);
  }

  async exchangeGenome(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/genome`);
  }

  async exchangeSimilar(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/similar`);
  }

  async projects() {
    return this.get("/v1/projects");
  }

  async project(id: string) {
    return this.get(`/v1/projects/${encodeURIComponent(id)}`);
  }

  async projectAlerts(id: string) {
    return this.get(`/v1/projects/${encodeURIComponent(id)}/alerts`);
  }

  async claimProject(id: string, claimantLabel?: string) {
    const res = await fetch(this.url(`/v1/projects/${encodeURIComponent(id)}/claim`), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ claimant_label: claimantLabel }),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async projectProfile(id: string, body: Record<string, string>) {
    const res = await fetch(this.url(`/v1/projects/${encodeURIComponent(id)}/profile`), {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async exchangeClaims(id: string) {
    return this.get(`/v1/ecosystems/${encodeURIComponent(id)}/claims`);
  }

  async mcpManifest() {
    return this.get("/v1/mcp");
  }

  async mcpCall(name: string, args: Record<string, unknown> = {}) {
    const res = await fetch(this.url("/v1/mcp"), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "tools/call",
        params: { name, arguments: args },
      }),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async exchangeHistory(id: string, range = "30d") {
    return this.get(
      `/v1/ecosystems/${encodeURIComponent(id)}/history?range=${encodeURIComponent(range)}`
    );
  }

  async twin(id: string) {
    return this.get(`/v2/ecosystems/${encodeURIComponent(id)}/twin`);
  }

  async simulate(body: Record<string, unknown>) {
    const res = await fetch(this.url("/v2/simulations"), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async copilot(question: string, ecosystemId?: string) {
    const res = await fetch(this.url("/v2/copilot"), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ question, ecosystem_id: ecosystemId }),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async twinReplay(id: string, range = "24h") {
    return this.get(
      `/v2/ecosystems/${encodeURIComponent(id)}/twin/replay?range=${encodeURIComponent(range)}`
    );
  }

  async whaleBehaviour(id: string) {
    return this.get(`/v2/ecosystems/${encodeURIComponent(id)}/whales`);
  }

  async compareTwins(a: string, b: string) {
    return this.get(
      `/v2/ecosystems/compare?ids=${encodeURIComponent(a)},${encodeURIComponent(b)}`
    );
  }

  async briefing() {
    return this.get("/v2/briefing");
  }

  async ask(question: string, tokenId?: string) {
    const res = await fetch(this.url("/v1/ai/query"), {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ question, token_id: tokenId }),
    });
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }

  async branding() {
    return this.get("/v1/branding");
  }

  async watchlist() {
    return this.get("/v1/watchlist");
  }

  private async get(path: string) {
    const res = await fetch(this.url(path));
    if (!res.ok) throw new Error(`MemeCoin OS API ${res.status}`);
    return res.json();
  }
}
