"use client";

import { api } from "@/lib/api";
import { compactInt, compactUsd, formatPrice, pct, riskTone, scoreTone } from "@/lib/format";
import type { Overview, TokenCard } from "@/lib/types";
import { DataStateBadge, ErrorState, Metric } from "@/components/ui";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useMemo, useState } from "react";
import {
  Bar,
  BarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

export default function OverviewPage() {
  const router = useRouter();
  const [data, setData] = useState<Overview | null>(null);
  const [narratives, setNarratives] = useState(0);
  const [candidates, setCandidates] = useState(0);
  const [alerts, setAlerts] = useState(0);
  const [freshness, setFreshness] = useState<string>("—");
  const [chatLive, setChatLive] = useState("—");
  const [error, setError] = useState<string | null>(null);
  const [q, setQ] = useState("");
  const [watchIds, setWatchIds] = useState<string[]>([]);
  const [watchOnly, setWatchOnly] = useState(false);

  useEffect(() => {
    Promise.all([
      api.overview(),
      api.narratives().catch(() => ({ narratives: [] })),
      api.discovery().catch(() => ({ candidates: [] })),
      api.alerts().catch(() => ({ alerts: [] })),
      api.status().catch(() => ({})),
      api.community().catch(() => ({ chat: { live: false, messages: 0 } })),
      api.watchlist().catch(() => ({ token_ids: [] as string[] })),
    ])
      .then(([ov, n, d, a, h, comm, w]) => {
        setData(ov);
        setNarratives((n.narratives ?? []).length);
        setCandidates((d.candidates ?? []).length);
        setAlerts((a.alerts ?? []).length);
        setChatLive(comm.chat?.live ? `live · ${comm.chat.messages ?? 0}` : "—");
        setWatchIds(w.token_ids ?? []);
        const health = h as { postgres?: { ok?: boolean }; tokens?: { live?: number }; degraded?: boolean };
        if (health.degraded) setFreshness("degraded");
        else if (health.postgres?.ok && (health.tokens?.live ?? 0) > 0) setFreshness("live");
        else setFreshness("unknown");
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  const matches = useMemo(() => {
    if (!data || !q.trim()) return [];
    const s = q.trim().toLowerCase();
    return data.tokens.filter((t) => {
      const c = (t.token.contract ?? "").toLowerCase();
      return (
        t.token.id.toLowerCase().includes(s) ||
        t.token.symbol.toLowerCase().includes(s) ||
        t.token.name.toLowerCase().includes(s) ||
        c.includes(s)
      );
    });
  }, [data, q]);

  const visible = useMemo(() => {
    if (!data) return [];
    if (!watchOnly || watchIds.length === 0) return data.tokens;
    return data.tokens.filter((t) => watchIds.includes(t.token.id));
  }, [data, watchOnly, watchIds]);

  function explore(id?: string) {
    const target = id ?? matches[0]?.token.id;
    if (target) router.push(`/twin/${target}`);
  }

  if (error) return <ErrorState message={error} />;
  if (!data) return <div className="text-mute text-sm">Indexing ecosystems…</div>;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">MEMECOIN OS 2.0</div>
        <h1 className="text-3xl mt-1">Intelligence infrastructure for the memecoin economy.</h1>
        <p className="text-mute mt-2 max-w-2xl text-sm">
          Understand ecosystems, narratives, risks and market structure through live Digital Twins
          and evidence-grounded AI. Nobody pays to change a score.
        </p>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-6 gap-3">
        <Metric label="Ecosystems monitored" value={String(data.tracked_tokens)} />
        <Metric
          label="Narratives detected"
          value={String(narratives)}
          hint="Registry tags. Mention firehose stays not connected without a licence."
        />
        <Metric label="Signals generated" value={String(alerts)} hint="Persisted alerts only. Not buy signals." />
        <Metric label="Digital Twins" value={String(data.tracked_tokens)} />
        <Metric label="Data freshness" value={freshness} />
        <Metric label="Community chat" value={chatLive} hint="First-party rooms are live. Mention firehose is a separate licence." />
      </div>

      <div className="panel p-5 space-y-3">
        <div className="kicker">Explore an ecosystem</div>
        <div className="flex flex-col sm:flex-row gap-2">
          <input
            className="flex-1 bg-void border border-line rounded-lg px-3 py-2 text-sm"
            placeholder="Token / contract / ecosystem id"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") explore();
            }}
          />
          <button
            className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm font-medium disabled:opacity-40"
            disabled={!matches[0]}
            onClick={() => explore()}
          >
            Explore Intelligence
          </button>
        </div>
        {q.trim() && matches.length === 0 ? (
          <div className="text-xs text-mute">No tracked match. Discovery and Onboard add UNVERIFIED only.</div>
        ) : null}
        {matches.length > 0 && q.trim() ? (
          <ul className="text-sm space-y-1">
            {matches.slice(0, 6).map((t) => (
              <li key={t.token.id}>
                <button className="text-phosphor font-mono text-xs" onClick={() => explore(t.token.id)}>
                  {t.token.symbol} · {t.token.id}
                </button>
              </li>
            ))}
          </ul>
        ) : null}
      </div>

      <div className="grid grid-cols-3 gap-3">
        <div className="panel p-4 col-span-2">
          <div className="kicker mb-4">Ecosystem health</div>
          <div className="h-56">
            <ResponsiveContainer>
              <BarChart data={visible.map((t) => ({ name: t.token.symbol, health: Number(t.health.toFixed(1)), risk: Number(t.risk.toFixed(1)) }))}>
                <XAxis dataKey="name" stroke="#8b95a8" fontSize={12} />
                <YAxis stroke="#8b95a8" fontSize={12} />
                <Tooltip contentStyle={{ background: "#0d1018", border: "1px solid #1c2333" }} />
                <Bar dataKey="health" fill="#3ee58a" radius={[4, 4, 0, 0]} />
                <Bar dataKey="risk" fill="#ff5d73" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>
        <div className="panel p-4 space-y-4">
          <div>
            <div className="kicker">Highest health</div>
            <ul className="mt-3 space-y-2">
              {data.highest_health.map((r) => (
                <li key={r.id} className="flex justify-between text-sm">
                  <Link href={`/tokens/${r.id}`} className="hover:text-phosphor">
                    {r.symbol}
                  </Link>
                  <span className={`font-mono ${scoreTone(r.value)}`}>{r.value.toFixed(0)}</span>
                </li>
              ))}
            </ul>
          </div>
          <div>
            <div className="kicker">Highest risk</div>
            <ul className="mt-3 space-y-2">
              {data.highest_risk.map((r) => (
                <li key={r.id} className="flex justify-between text-sm">
                  <Link href={`/tokens/${r.id}`} className="hover:text-phosphor">
                    {r.symbol}
                  </Link>
                  <span className={`font-mono ${riskTone(r.level)}`}>
                    {r.value.toFixed(0)} {r.level}
                  </span>
                </li>
              ))}
            </ul>
          </div>
          <div className="text-xs text-mute">Discovery candidates (not verified): {candidates}</div>
          <Link href="/exchange" className="block text-xs text-phosphor mt-3">
            Exchange API · widget · MCP →
          </Link>
        </div>
      </div>

      <TokenTable
        tokens={visible}
        watchOnly={watchOnly}
        watchCount={watchIds.length}
        onWatchOnly={setWatchOnly}
      />
      <p className="text-xs text-mute">{data.disclaimer}</p>
    </div>
  );
}

function TokenTable({
  tokens,
  watchOnly,
  watchCount,
  onWatchOnly,
}: {
  tokens: TokenCard[];
  watchOnly: boolean;
  watchCount: number;
  onWatchOnly: (v: boolean) => void;
}) {
  return (
    <div className="panel overflow-hidden">
      <div className="flex justify-between items-center px-4 py-2 border-b border-line">
        <label className="flex items-center gap-2 text-xs text-mute">
          <input
            type="checkbox"
            checked={watchOnly}
            disabled={watchCount === 0}
            onChange={(e) => onWatchOnly(e.target.checked)}
          />
          Watchlist only{watchCount > 0 ? ` · ${watchCount}` : " · empty — set in Settings"}
        </label>
        <a href="/v1/overview.csv" className="text-xs text-signal font-mono">
          Export CSV
        </a>
      </div>
      <table className="w-full text-sm">
        <thead className="text-mute font-mono text-[11px] uppercase tracking-wider border-b border-line">
          <tr>
            {["Token", "Chain", "Price", "Health", "Risk", "Mkt cap", "Volume", "24h", "Holders", "State"].map((h) => (
              <th key={h} className="text-left font-medium px-4 py-3">
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {tokens.map((t) => (
            <tr key={t.token.id} className="border-b border-line/70 hover:bg-white/[0.03]">
              <td className="px-4 py-3">
                <Link href={`/tokens/${t.token.id}`} className="hover:text-phosphor">
                  <div className="font-medium">{t.token.symbol}</div>
                  <div className="text-xs text-mute">{t.token.name}</div>
                </Link>
              </td>
              <td className="px-4 py-3 text-mute">{t.token.primary_chain}</td>
              <td className="px-4 py-3 font-mono">{formatPrice(t.price_usd, t.market_state)}</td>
              <td className={`px-4 py-3 font-mono ${scoreTone(t.health)}`}>{t.health.toFixed(0)}</td>
              <td className={`px-4 py-3 font-mono ${riskTone(t.risk_level)}`}>
                {t.risk.toFixed(0)} {t.risk_level}
              </td>
              <td className="px-4 py-3 font-mono">{compactUsd(t.market_cap_usd)}</td>
              <td className="px-4 py-3 font-mono">{compactUsd(t.volume_24h_usd)}</td>
              <td className={`px-4 py-3 font-mono ${t.change_24h_pct >= 0 ? "text-phosphor" : "text-danger"}`}>
                {pct(t.change_24h_pct)}
              </td>
              <td className="px-4 py-3 font-mono">
                {t.holders_state && t.holders_state !== "live" && t.holders_state !== "recent" && t.holders_state !== "simulated"
                  ? "—"
                  : compactInt(t.holders)}
              </td>
              <td className="px-4 py-3">
                <DataStateBadge state={t.data_state ?? t.market_state} />
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
