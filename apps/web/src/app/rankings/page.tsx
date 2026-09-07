"use client";

import { api } from "@/lib/api";
import { riskTone, scoreTone } from "@/lib/format";
import type { Overview } from "@/lib/types";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useMemo, useState } from "react";

type Rankings = {
  health: { id: string; symbol: string; value: number }[];
  momentum: { id: string; symbol: string; value: number }[];
  risk: { id: string; symbol: string; value: number; level: string }[];
  development: { id: string; symbol: string; value: number }[];
  community: { id: string; symbol: string; value: number }[];
  liquidity: { id: string; symbol: string; value: number }[];
};

export default function RankingsPage() {
  const [data, setData] = useState<Rankings | null>(null);
  const [cards, setCards] = useState<Overview["tokens"]>([]);
  const [error, setError] = useState<string | null>(null);
  const [chain, setChain] = useState("all");
  const [risk, setRisk] = useState("all");
  const [state, setState] = useState("all");

  useEffect(() => {
    Promise.all([api.rankings(), api.overview()])
      .then(([r, o]) => {
        setData(r);
        setCards(o.tokens);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  const allowed = useMemo(() => {
    const ids = new Set(
      cards
        .filter((t) => {
          if (chain !== "all" && t.token.primary_chain !== chain) return false;
          if (risk !== "all" && t.risk_level.toLowerCase() !== risk) return false;
          if (state !== "all" && (t.data_state ?? "").toLowerCase() !== state) return false;
          return true;
        })
        .map((t) => t.token.id)
    );
    return ids;
  }, [cards, chain, risk, state]);

  if (error) return <ErrorState message={error} />;
  if (!data) return <div className="text-mute text-sm">Computing rankings…</div>;

  const keep = <T extends { id: string }>(rows: T[]) => rows.filter((r) => allowed.has(r.id));
  const boards: [string, { id: string; symbol: string; value: number; level?: string }[]][] = [
    ["Health", keep(data.health)],
    ["Momentum", keep(data.momentum)],
    ["Risk", keep(data.risk)],
    ["Development", keep(data.development)],
    ["Community", keep(data.community)],
    ["Liquidity", keep(data.liquidity)],
  ];
  const chains = Array.from(new Set(cards.map((t) => t.token.primary_chain)));
  const stateOf = (id: string) => cards.find((c) => c.token.id === id)?.data_state;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">MemeCoin 100</div>
        <h1 className="text-3xl mt-1">Rankings</h1>
        <p className="text-sm text-mute mt-2">
          Never market-cap only. Health, momentum, risk, development, community and liquidity are ranked separately.
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
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {boards.map(([title, rows]) => (
          <div key={title} className="panel p-4">
            <div className="kicker mb-3">{title}</div>
            <ol className="space-y-2">
              {rows.map((r, i) => (
                <li key={r.id} className="flex justify-between text-sm gap-2">
                  <Link href={`/tokens/${r.id}`} className="hover:text-phosphor min-w-0 truncate">
                    <span className="text-mute mr-2">{i + 1}</span>
                    {r.symbol}
                  </Link>
                  <span className="flex items-center gap-2 shrink-0">
                    <DataStateBadge state={stateOf(r.id)} />
                    <span className={`font-mono ${title === "Risk" ? riskTone(r.level ?? "") : scoreTone(r.value)}`}>
                      {r.value.toFixed(0)}
                      {r.level ? ` ${r.level}` : ""}
                    </span>
                  </span>
                </li>
              ))}
            </ol>
          </div>
        ))}
      </div>
    </div>
  );
}
