"use client";

import { api } from "@/lib/api";
import { compactInt, compactUsd, formatPrice, pct, riskTone, scoreTone } from "@/lib/format";
import type { Overview } from "@/lib/types";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useState } from "react";

export default function TokensPage() {
  const [data, setData] = useState<Overview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [chain, setChain] = useState("all");
  const [risk, setRisk] = useState("all");
  const [state, setState] = useState("all");
  const [watchIds, setWatchIds] = useState<string[]>([]);
  const [watchOnly, setWatchOnly] = useState(false);

  useEffect(() => {
    Promise.all([
      api.overview(),
      api.watchlist().catch(() => ({ token_ids: [] as string[] })),
    ])
      .then(([ov, w]) => {
        setData(ov);
        setWatchIds(w.token_ids ?? []);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;
  if (!data) return <div className="text-mute text-sm">Loading registry…</div>;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Token-agnostic registry</div>
        <h1 className="text-3xl mt-1">Ecosystems</h1>
        <p className="text-sm text-mute mt-2">
          Seeds ship as YAML. Track from Discovery or Onboard writes UNVERIFIED definitions to Postgres so they survive API restarts.
        </p>
      </div>
      <div className="flex flex-wrap gap-2 text-xs">
        <select className="bg-void border border-line rounded px-2 py-1" value={chain} onChange={(e) => setChain(e.target.value)}>
          <option value="all">all chains</option>
          {Array.from(new Set(data.tokens.map((t) => t.token.primary_chain))).map((c) => (
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
        <label className="flex items-center gap-2 text-mute">
          <input
            type="checkbox"
            checked={watchOnly}
            disabled={watchIds.length === 0}
            onChange={(e) => setWatchOnly(e.target.checked)}
          />
          watchlist only{watchIds.length > 0 ? ` · ${watchIds.length}` : ""}
        </label>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {data.tokens.filter((t) => {
          if (watchOnly && !watchIds.includes(t.token.id)) return false;
          if (chain !== "all" && t.token.primary_chain !== chain) return false;
          if (risk !== "all" && t.risk_level.toLowerCase() !== risk) return false;
          if (state !== "all" && (t.data_state ?? "").toLowerCase() !== state) return false;
          return true;
        }).map((t) => (
          <Link key={t.token.id} href={`/tokens/${t.token.id}`} className="panel p-5 hover:border-phosphor/40 transition-colors">
            <div className="flex justify-between items-start">
              <div>
                <div className="font-mono text-phosphor text-xs">{t.token.symbol}</div>
                <div className="text-lg">{t.token.name}</div>
              </div>
              <div className={`font-mono text-2xl ${scoreTone(t.health)}`}>{t.health.toFixed(0)}</div>
            </div>
            <div className="mt-2">
              <DataStateBadge state={t.data_state} />
            </div>
            <div className="mt-4 flex flex-wrap gap-1">
              {t.token.narratives.map((n) => (
                <span key={n} className="text-[10px] uppercase tracking-wide border border-line rounded px-1.5 py-0.5 text-mute">
                  {n}
                </span>
              ))}
            </div>
            <dl className="mt-4 grid grid-cols-2 gap-2 text-xs">
              <div>
                <dt className="text-mute">Price</dt>
                <dd className="font-mono">{formatPrice(t.price_usd, t.market_state)}</dd>
              </div>
              <div>
                <dt className="text-mute">Chain</dt>
                <dd>{t.token.primary_chain}</dd>
              </div>
              <div>
                <dt className="text-mute">Risk</dt>
                <dd className={riskTone(t.risk_level)}>{t.risk.toFixed(0)} {t.risk_level}</dd>
              </div>
              <div>
                <dt className="text-mute">Mkt cap</dt>
                <dd className="font-mono">{compactUsd(t.market_cap_usd)}</dd>
              </div>
              <div>
                <dt className="text-mute">24h</dt>
                <dd className={`font-mono ${t.change_24h_pct >= 0 ? "text-phosphor" : "text-danger"}`}>{pct(t.change_24h_pct)}</dd>
              </div>
              <div>
                <dt className="text-mute">Holders</dt>
                <dd className="font-mono">{compactInt(t.holders)}</dd>
              </div>
              <div>
                <dt className="text-mute">Verification</dt>
                <dd>{t.token.verification.replace(/_/g, " ")}</dd>
              </div>
            </dl>
          </Link>
        ))}
      </div>
    </div>
  );
}
