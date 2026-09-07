"use client";

import { api } from "@/lib/api";
import type { Overview } from "@/lib/types";
import { useEffect, useMemo, useState } from "react";

const HOST = "https://memecoin-os.web.app";
const API = "https://memecoin-os.web.app";

export default function ExchangeIntegratorsPage() {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [id, setId] = useState("pepe");

  useEffect(() => {
    api
      .overview()
      .then((o) => {
        setOverview(o);
        if (o.tokens[0]?.token.id) setId(o.tokens[0].token.id);
      })
      .catch(() => setOverview(null));
  }, []);

  const iframe = `<iframe
  src="${HOST}/embed/${id}/"
  width="380"
  height="340"
  style="border:0;border-radius:12px;background:#070910"
  title="MemeCoin OS Intelligence"
></iframe>`;

  const widget = `<div id="mcos-card"></div>
<script src="${HOST}/widget.js"></script>
<script>
  MemeCoinOS.embed({ id: "${id}", target: "#mcos-card" });
</script>`;

  const rest = useMemo(
    () => [
      `GET ${API}/v1/ecosystems`,
      `GET ${API}/v1/ecosystems/${id}`,
      `GET ${API}/v1/ecosystems/${id}/asset`,
      `GET ${API}/v1/ecosystems/${id}/twin`,
      `GET ${API}/v1/ecosystems/${id}/risk`,
      `GET ${API}/v1/ecosystems/${id}/narratives`,
      `GET ${API}/v1/ecosystems/${id}/genome`,
      `GET ${API}/v1/ecosystems/${id}/similar`,
      `GET ${API}/v1/ecosystems/${id}/history?range=30d`,
      `GET ${API}/v1/ecosystems/${id}/claims`,
      `GET ${API}/v1/mcp`,
      `POST ${API}/v1/mcp`,
      `GET ${API}/v1/discovery`,
      `GET ${API}/v1/narratives`,
      `GET ${API}/v1/alerts`,
      `GET ${API}/v1/health`,
    ],
    [id]
  );

  return (
    <div className="space-y-6 max-w-4xl">
      <div>
        <div className="kicker">MemeCoin OS Intelligence API</div>
        <h1 className="text-3xl mt-1">Exchange widget and REST</h1>
        <p className="text-sm text-mute mt-2">
          Drop-in card plus additive <code className="font-mono">/v1/ecosystems*</code> for listing
          pages. Observable state only. Not a trade signal. EXECUTE stays off.{" "}
          <span className="text-ink">VERIFIED ≠ SAFE.</span> Sources without a feed stay blank, not zero.
        </p>
      </div>

      <div className="panel p-4">
        <label className="kicker block mb-2">Preview asset</label>
        <select
          className="bg-void border border-line rounded-lg px-3 py-2 text-sm font-mono"
          value={id}
          onChange={(e) => setId(e.target.value)}
        >
          {(overview?.tokens ?? []).map((t) => (
            <option key={t.token.id} value={t.token.id}>
              {t.token.symbol} · {t.token.id}
            </option>
          ))}
        </select>
        <div className="mt-4">
          <iframe
            src={`/embed/${id}/`}
            width={380}
            height={340}
            title="MemeCoin OS Intelligence"
            className="rounded-xl bg-[#070910] max-w-full"
            style={{ border: 0 }}
          />
        </div>
      </div>

      <Snippet title="Iframe" code={iframe} />
      <Snippet title="JavaScript widget" code={widget} />
      <Snippet
        title="MCP (Cursor / Claude)"
        code={`# HTTP (live)
GET ${API}/v1/mcp
POST ${API}/v1/mcp
{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_risk","arguments":{"id":"${id}"}}}

# Stdio
MEMECOIN_OS_API=${API} node packages/mcp/server.mjs

# .cursor/mcp.json
{
  "mcpServers": {
    "memecoin-os": {
      "command": "node",
      "args": ["packages/mcp/server.mjs"]
    }
  }
}`}
      />

      <div className="panel p-5 space-y-3">
        <div className="kicker">REST envelope</div>
        <p className="text-sm text-mute">
          Every exchange route returns <code className="font-mono">data</code>,{" "}
          <code className="font-mono">timestamp</code>, <code className="font-mono">freshness</code>,{" "}
          <code className="font-mono">confidence</code>, <code className="font-mono">evidence</code>,{" "}
          <code className="font-mono">powered_by</code>, and <code className="font-mono">execute: false</code>.
        </p>
        <ul className="text-xs font-mono text-mute space-y-1">
          {rest.map((line) => (
            <li key={line}>{line}</li>
          ))}
        </ul>
        <p className="text-xs text-mute">
          TypeScript: <code>@memecoin-os/sdk</code> methods{" "}
          <code className="font-mono">exchangeEcosystems()</code>,{" "}
          <code className="font-mono">exchangeAsset(id)</code>,{" "}
          <code className="font-mono">exchangeTwin(id)</code>. Python:{" "}
          <code className="font-mono">exchange_ecosystems()</code>,{" "}
          <code className="font-mono">exchange_asset(id)</code>.
        </p>
      </div>

      <p className="text-[11px] text-mute">
        Powered by MemeCoin OS. Similarity is structural, not “will pump like X”. Paying never
        changes a score.
      </p>
    </div>
  );
}

function Snippet({ title, code }: { title: string; code: string }) {
  return (
    <div className="panel p-5">
      <div className="kicker mb-2">{title}</div>
      <pre className="text-xs text-mute overflow-auto whitespace-pre-wrap font-mono">{code}</pre>
    </div>
  );
}
