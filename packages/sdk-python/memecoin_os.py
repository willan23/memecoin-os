"""MemeCoin OS REST client (read / analyze / recommend). EXECUTE is never exposed."""

from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any


class MemeCoinOsClient:
    def __init__(self, base_url: str = "http://127.0.0.1:8080", api_key: str | None = None) -> None:
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key

    def _headers(self) -> dict[str, str]:
        headers = {"accept": "application/json"}
        if self.api_key:
            headers["authorization"] = f"Bearer {self.api_key}"
        return headers

    def _get(self, path: str) -> Any:
        req = urllib.request.Request(self.base_url + path, headers=self._headers())
        try:
            with urllib.request.urlopen(req, timeout=30) as res:
                return json.loads(res.read().decode())
        except urllib.error.HTTPError as e:
            raise RuntimeError(f"MemeCoin OS API {e.code} {path}") from e

    def _send(self, method: str, path: str, body: dict[str, Any] | None = None) -> Any:
        data = None if body is None else json.dumps(body).encode()
        headers = self._headers()
        if data is not None:
            headers["content-type"] = "application/json"
        req = urllib.request.Request(self.base_url + path, data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=30) as res:
                raw = res.read()
                return json.loads(raw.decode()) if raw else None
        except urllib.error.HTTPError as e:
            raise RuntimeError(f"MemeCoin OS API {e.code} {path}") from e

    def health(self) -> Any:
        return self._get("/health")

    def overview(self) -> Any:
        return self._get("/v1/overview")

    def tokens(self) -> Any:
        return self._get("/v1/tokens")

    def token(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}")

    def history(self, token_id: str, range: str = "30d") -> Any:
        q = urllib.parse.quote(range)
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/history?range={q}")

    def baselines(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/baselines")

    def pools(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/pools")

    def wallets(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/wallets")

    def indexer(self) -> Any:
        return self._get("/v1/indexer")

    def transfers(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/transfers")

    def whale_events(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/whale-events")

    def clusters(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/clusters")

    def discovery(self) -> Any:
        return self._get("/v1/discovery")

    def narratives(self) -> Any:
        return self._get("/v1/narratives")

    def genome_clusters(self) -> Any:
        return self._get("/v1/genome-clusters")

    def verification(self, token_id: str) -> Any:
        return self._get(f"/v1/tokens/{urllib.parse.quote(token_id)}/verification")

    def compare(self, ids: list[str]) -> Any:
        return self._get("/v1/compare?ids=" + ",".join(urllib.parse.quote(i) for i in ids))

    def rankings(self) -> Any:
        return self._get("/v1/rankings")

    def alerts(self) -> Any:
        return self._get("/v1/alerts")

    def flags(self) -> Any:
        return self._get("/v1/flags")

    def ask(self, question: str, token_id: str | None = None) -> Any:
        return self._send("POST", "/v1/ai/query", {"question": question, "token_id": token_id})

    def research(self, question: str, token_id: str | None = None) -> Any:
        return self._send("POST", "/v1/research", {"question": question, "token_id": token_id})

    def billing(self) -> Any:
        return self._get("/v1/billing")

    def queue(self) -> Any:
        return self._get("/v1/queue")

    def tenants(self) -> Any:
        return self._get("/v1/tenants")

    def sso(self) -> Any:
        return self._get("/v1/auth/sso")

    def discovery_v2(self) -> Any:
        return self._get("/v2/discovery")

    def ecosystems(self) -> Any:
        return self._get("/v2/ecosystems")

    def exchange_health(self) -> Any:
        return self._get("/v1/health")

    def exchange_ecosystems(self) -> Any:
        return self._get("/v1/ecosystems")

    def exchange_ecosystem(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}")

    def exchange_asset(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/asset")

    def exchange_twin(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/twin")

    def exchange_risk(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/risk")

    def exchange_narratives(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/narratives")

    def exchange_genome(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/genome")

    def exchange_similar(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/similar")

    def projects(self) -> Any:
        return self._get("/v1/projects")

    def project(self, token_id: str) -> Any:
        return self._get(f"/v1/projects/{urllib.parse.quote(token_id)}")

    def project_alerts(self, token_id: str) -> Any:
        return self._get(f"/v1/projects/{urllib.parse.quote(token_id)}/alerts")

    def claim_project(self, token_id: str, claimant_label: str | None = None) -> Any:
        return self._send("POST", f"/v1/projects/{urllib.parse.quote(token_id)}/claim", {"claimant_label": claimant_label})

    def project_profile(self, token_id: str, **links: str) -> Any:
        return self._send("PATCH", f"/v1/projects/{urllib.parse.quote(token_id)}/profile", links)

    def exchange_claims(self, ecosystem_id: str) -> Any:
        return self._get(f"/v1/ecosystems/{urllib.parse.quote(ecosystem_id)}/claims")

    def mcp_manifest(self) -> Any:
        return self._get("/v1/mcp")

    def mcp_call(self, name: str, **arguments: Any) -> Any:
        return self._send(
            "POST",
            "/v1/mcp",
            {"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": name, "arguments": arguments}},
        )

    def exchange_history(self, ecosystem_id: str, range: str = "30d") -> Any:
        q = urllib.parse.quote(ecosystem_id)
        return self._get(f"/v1/ecosystems/{q}/history?range={urllib.parse.quote(range)}")

    def twin(self, ecosystem_id: str) -> Any:
        return self._get(f"/v2/ecosystems/{urllib.parse.quote(ecosystem_id)}/twin")

    def simulate(self, ecosystem_id: str, **assumptions: Any) -> Any:
        body = {"ecosystem_id": ecosystem_id, **assumptions}
        return self._send("POST", "/v2/simulations", body)

    def copilot(self, question: str, ecosystem_id: str | None = None) -> Any:
        return self._send("POST", "/v2/copilot", {"question": question, "ecosystem_id": ecosystem_id})

    def twin_replay(self, ecosystem_id: str, range: str = "24h") -> Any:
        q = urllib.parse.quote(ecosystem_id)
        return self._get(f"/v2/ecosystems/{q}/twin/replay?range={urllib.parse.quote(range)}")

    def whale_behaviour(self, ecosystem_id: str) -> Any:
        q = urllib.parse.quote(ecosystem_id)
        return self._get(f"/v2/ecosystems/{q}/whales")

    def compare_twins(self, a: str, b: str) -> Any:
        return self._get(f"/v2/ecosystems/compare?ids={urllib.parse.quote(a)},{urllib.parse.quote(b)}")

    def briefing(self) -> Any:
        return self._get("/v2/briefing")

    def branding(self) -> Any:
        return self._get("/v1/branding")

    def watchlist(self) -> Any:
        return self._get("/v1/watchlist")
