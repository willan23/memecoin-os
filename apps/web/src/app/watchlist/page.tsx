"use client";

import { api } from "@/lib/api";
import { compactInt, compactUsd, formatPrice, pct, riskTone, scoreTone } from "@/lib/format";
import type { TokenCard } from "@/lib/types";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useMemo, useState } from "react";

export default function WatchlistPage() {
  const [tokens, setTokens] = useState<TokenCard[]>([]);
  const [ids, setIds] = useState<string[]>([]);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([
      api.overview(),
      api.watchlist().catch(() => ({ token_ids: [] as string[], note: "Watchlist unavailable (Postgres)." })),
    ])
      .then(([ov, w]) => {
        setTokens(ov.tokens);
        setIds(w.token_ids ?? []);
        setNote(w.note ?? null);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  const watched = useMemo(
    () => tokens.filter((t) => ids.includes(t.token.id)),
    [tokens, ids]
  );

  if (error) return <ErrorState message={error} />;
  if (!tokens.length && !note) return <div className="text-mute text-sm">Loading watchlist…</div>;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Personal filter</div>
        <h1 className="text-3xl mt-1">Watchlist</h1>
        <p className="text-sm text-mute mt-2">
          A list you own. Snapshots stay shared. Server-side isolation only when AUTH_REQUIRED is on.
          Not a forked Twin. Not a ranking.
        </p>
      </div>

      {ids.length === 0 ? (
        <div className="panel p-5 text-sm text-mute">
          Empty. Pick ecosystems in{" "}
          <Link href="/settings" className="text-phosphor underline">
            Settings
          </Link>
          .
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {watched.map((t) => (
            <Link
              key={t.token.id}
              href={`/tokens/${t.token.id}`}
              className="panel p-5 hover:border-phosphor/40 transition-colors"
            >
              <div className="flex justify-between items-start">
                <div>
                  <div className="font-mono text-phosphor text-xs">{t.token.symbol}</div>
                  <div className="text-lg">{t.token.name}</div>
                  <div className="text-[11px] text-mute font-mono mt-1">{t.token.id}</div>
                </div>
                <div className={`font-mono text-2xl ${scoreTone(t.health)}`}>{t.health.toFixed(0)}</div>
              </div>
              <div className="mt-2">
                <DataStateBadge state={t.data_state} />
              </div>
              <dl className="mt-4 grid grid-cols-2 gap-2 text-xs">
                <div>
                  <dt className="text-mute">Price</dt>
                  <dd className="font-mono">{formatPrice(t.price_usd, t.market_state)}</dd>
                </div>
                <div>
                  <dt className="text-mute">Risk</dt>
                  <dd className={`font-mono ${riskTone(t.risk_level)}`}>
                    {t.risk.toFixed(0)} {t.risk_level}
                  </dd>
                </div>
                <div>
                  <dt className="text-mute">Mkt cap</dt>
                  <dd className="font-mono">{compactUsd(t.market_cap_usd)}</dd>
                </div>
                <div>
                  <dt className="text-mute">24h</dt>
                  <dd className={`font-mono ${t.change_24h_pct >= 0 ? "text-phosphor" : "text-danger"}`}>
                    {pct(t.change_24h_pct)}
                  </dd>
                </div>
                <div>
                  <dt className="text-mute">Holders</dt>
                  <dd className="font-mono">
                    {t.holders_state &&
                    t.holders_state !== "live" &&
                    t.holders_state !== "recent" &&
                    t.holders_state !== "simulated"
                      ? "—"
                      : compactInt(t.holders)}
                  </dd>
                </div>
              </dl>
            </Link>
          ))}
        </div>
      )}

      {ids.length > 0 && watched.length === 0 ? (
        <div className="text-sm text-mute">
          Saved ids are not in the live catalog: {ids.join(", ")}
        </div>
      ) : null}

      {note ? <p className="text-xs text-mute">{note}</p> : null}
    </div>
  );
}
