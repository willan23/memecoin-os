"use client";

import { api } from "@/lib/api";
import { compactUsd, scoreTone } from "@/lib/format";
import type { Overview, TokenSnapshot } from "@/lib/types";
import { DataStateBadge, ErrorState } from "@/components/ui";
import { useEffect, useMemo, useState } from "react";

const COLORS = ["#3ee58a", "#4da6ff", "#f0c14b", "#ff5d73"];

export default function ComparePage() {
  const [catalog, setCatalog] = useState<Overview["tokens"]>([]);
  const [ids, setIds] = useState<string[]>([]);
  const [tokens, setTokens] = useState<TokenSnapshot[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [chain, setChain] = useState("all");
  const [risk, setRisk] = useState("all");
  const [state, setState] = useState("all");

  useEffect(() => {
    api
      .overview()
      .then((o) => {
        setCatalog(o.tokens);
        setIds((cur) => (cur.length ? cur : o.tokens.slice(0, 4).map((t) => t.token.id)));
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  useEffect(() => {
    if (!ids.length) return;
    api
      .compare(ids)
      .then((r) => setTokens(r.tokens))
      .catch((e) => setError(String(e.message ?? e)));
  }, [ids]);

  const filteredCatalog = useMemo(() => {
    return catalog.filter((t) => {
      if (chain !== "all" && t.token.primary_chain !== chain) return false;
      if (risk !== "all" && t.risk_level.toLowerCase() !== risk) return false;
      if (state !== "all" && (t.data_state ?? "").toLowerCase() !== state) return false;
      return true;
    });
  }, [catalog, chain, risk, state]);

  const radar = useMemo(() => {
    if (!tokens[0]) return [];
    return tokens[0].genome.dimensions.map((d) => {
      const row: Record<string, string | number> = { k: d.label };
      tokens.forEach((t) => {
        row[t.token.symbol] =
          t.genome.dimensions.find((x) => x.id === d.id)?.value ?? 0;
      });
      return row;
    });
  }, [tokens]);

  const chains = Array.from(new Set(catalog.map((t) => t.token.primary_chain)));

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Benchmark engine</div>
        <h1 className="text-3xl mt-1">Compare ecosystems</h1>
        <p className="text-sm text-mute mt-2">
          Normalized genome and health. Similar characteristics are not copy-paste strategies.
        </p>
      </div>
      <div className="flex flex-wrap gap-2 text-xs">
        <select className="bg-void border border-line rounded px-2 py-1" value={chain} onChange={(e) => setChain(e.target.value)}>
          <option value="all">all chains</option>
          {chains.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
        <select className="bg-void border border-line rounded px-2 py-1" value={risk} onChange={(e) => setRisk(e.target.value)}>
          <option value="all">all risk</option>
          {["low", "moderate", "high", "critical", "unknown"].map((r) => (
            <option key={r} value={r}>{r}</option>
          ))}
        </select>
        <select className="bg-void border border-line rounded px-2 py-1" value={state} onChange={(e) => setState(e.target.value)}>
          <option value="all">all data_state</option>
          {["live", "recent", "stale", "missing", "conflict", "simulated"].map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>
      <div className="flex gap-2 text-xs font-mono flex-wrap">
        {filteredCatalog.map((t) => (
          <button
            key={t.token.id}
            onClick={() =>
              setIds((cur) =>
                cur.includes(t.token.id) ? cur.filter((x) => x !== t.token.id) : [...cur, t.token.id]
              )
            }
            className={`border rounded px-2 py-1 ${ids.includes(t.token.id) ? "border-phosphor text-phosphor" : "border-line text-mute"}`}
          >
            {t.token.symbol}
          </button>
        ))}
      </div>
      <div className="flex flex-wrap gap-2">
        {tokens.map((t) => (
          <div key={t.token.id} className="flex items-center gap-2 text-xs">
            <span className="font-mono">{t.token.symbol}</span>
            <DataStateBadge state={t.data_state} />
          </div>
        ))}
      </div>
      <div className="panel p-4 space-y-3 max-h-96 overflow-auto">
        {radar.map((row) => (
          <div key={String(row.k)}>
            <div className="text-[11px] text-mute mb-1">{row.k}</div>
            <div className="grid gap-2" style={{ gridTemplateColumns: `repeat(${tokens.length || 1}, minmax(0, 1fr))` }}>
              {tokens.map((t, i) => {
                const v = Number(row[t.token.symbol] ?? 0);
                return (
                  <div key={t.token.id}>
                    <div className="flex justify-between font-mono text-[10px] text-mute mb-0.5">
                      <span>{t.token.symbol}</span>
                      <span>{v.toFixed(0)}</span>
                    </div>
                    <div className="h-1.5 rounded-full bg-white/5 overflow-hidden">
                      <div
                        className="h-full"
                        style={{ width: `${Math.max(2, Math.min(100, v))}%`, background: COLORS[i % COLORS.length] }}
                      />
                    </div>
                  </div>
                );
              })}
            </div>
          </div>
        ))}
      </div>
      <div className="panel overflow-auto">
        <table className="w-full text-sm">
          <thead className="text-mute text-[11px] uppercase font-mono border-b border-line">
            <tr>
              <th className="text-left px-3 py-2">Metric</th>
              {tokens.map((t) => (
                <th key={t.token.id} className="text-left px-3 py-2">{t.token.symbol}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {[
              ["Health", ...tokens.map((t) => t.scores.value.toFixed(0))],
              ["Risk", ...tokens.map((t) => `${t.risk.score.toFixed(0)} ${t.risk.level}`)],
              ["Market cap", ...tokens.map((t) => t.market.data_state && t.market.data_state !== "missing" ? compactUsd(t.market.market_cap_usd) : "—")],
              ["Liquidity", ...tokens.map((t) => t.liquidity.data_state && t.liquidity.data_state !== "missing" ? compactUsd(t.liquidity.liquidity_usd) : "—")],
              ["Holders", ...tokens.map((t) => t.onchain.data_state && t.onchain.data_state !== "missing" ? t.onchain.holders.toLocaleString() : "—")],
              ["Organicness", ...tokens.map((t) => t.social.data_state && t.social.data_state !== "missing" ? t.social.organicness.toFixed(2) : "—")],
              ["Dev activity", ...tokens.map((t) => t.development.data_state && t.development.data_state !== "missing" ? t.development.activity_score.toFixed(0) : "—")],
            ].map((row) => (
              <tr key={row[0]} className="border-b border-line/60">
                {row.map((c, i) => (
                  <td key={i} className={`px-3 py-2 ${i === 0 ? "text-mute" : "font-mono"}`}>
                    {i > 0 && row[0] === "Health" ? (
                      <span className={scoreTone(Number(c))}>{c}</span>
                    ) : (
                      c
                    )}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
