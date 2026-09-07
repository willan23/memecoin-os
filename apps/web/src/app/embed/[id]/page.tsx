"use client";

import { api } from "@/lib/api";
import { formatPrice, pct } from "@/lib/format";
import { DataStateBadge } from "@/components/ui";
import type { TokenSnapshot } from "@/lib/types";
import { useEntityId } from "@/lib/route-id";
import { useEffect, useState } from "react";

function present(state?: string | null) {
  const s = (state ?? "").toLowerCase();
  return s === "live" || s === "recent" || s === "stale" || s === "conflict" || s === "simulated";
}

function ago(iso: string) {
  const t = Date.parse(iso);
  if (!Number.isFinite(t)) return iso;
  const sec = Math.max(0, Math.round((Date.now() - t) / 1000));
  if (sec < 60) return `${sec}s ago`;
  if (sec < 3600) return `${Math.round(sec / 60)}m ago`;
  if (sec < 86400) return `${Math.round(sec / 3600)}h ago`;
  return iso.slice(0, 10);
}

/** Compact Twin card for an exchange listing page iframe. No EXECUTE. */
export default function EmbedPage() {
  const id = useEntityId();
  const [data, setData] = useState<TokenSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    api
      .token(id)
      .then(setData)
      .catch((e) => setError(String((e as Error).message ?? e)));
  }, [id]);

  if (error) {
    return (
      <div className="min-h-screen bg-[#070910] text-[#8b95a8] p-4 text-sm">
        {error}. A blank is not a zero.
      </div>
    );
  }
  if (!data) {
    return <div className="min-h-screen bg-[#070910] text-[#8b95a8] p-4 text-sm">Loading Twin…</div>;
  }

  const momentum = present(data.market.data_state) ? pct(data.market.change_24h_pct) : "—";
  const organic = present(data.social.data_state)
    ? `${Math.round(data.social.organicness * 100)}`
    : "—";
  const tags = data.token.narratives.length ? data.token.narratives.slice(0, 3).join(" · ") : "—";

  return (
    <div className="min-h-screen bg-[#070910] text-[#e8edf5] p-4 font-sans">
      <div className="text-[10px] tracking-[0.2em] text-[#3ee58a] font-mono">MEMECOIN OS INTELLIGENCE</div>
      <div className="flex justify-between items-start mt-2 gap-2">
        <div>
          <div className="text-xl">{data.token.name}</div>
          <div className="font-mono text-xs text-[#8b95a8]">{data.token.symbol}</div>
        </div>
        <DataStateBadge state={data.data_state} />
      </div>
      <dl className="grid grid-cols-2 gap-2 mt-4 text-sm">
        <div>
          <dt className="text-[10px] text-[#8b95a8] uppercase">Price</dt>
          <dd className="font-mono">{formatPrice(data.market.price_usd, data.market.data_state)}</dd>
        </div>
        <div>
          <dt className="text-[10px] text-[#8b95a8] uppercase">Health</dt>
          <dd className="font-mono">{data.scores.value.toFixed(0)}</dd>
        </div>
        <div>
          <dt className="text-[10px] text-[#8b95a8] uppercase">Risk</dt>
          <dd className="font-mono">
            {data.risk.score.toFixed(0)} · {data.risk.level}
          </dd>
        </div>
        <div>
          <dt className="text-[10px] text-[#8b95a8] uppercase">Momentum</dt>
          <dd className="font-mono">{momentum}</dd>
        </div>
        <div>
          <dt className="text-[10px] text-[#8b95a8] uppercase">Organicness</dt>
          <dd className="font-mono">{organic}</dd>
        </div>
        <div>
          <dt className="text-[10px] text-[#8b95a8] uppercase">Holders</dt>
          <dd className="font-mono">{present(data.onchain.data_state) ? data.onchain.holders : "—"}</dd>
        </div>
      </dl>
      <div className="mt-4">
        <div className="text-[10px] text-[#8b95a8] uppercase">Narrative</div>
        <div className="text-sm mt-1">{tags}</div>
      </div>
      <div className="mt-3 text-[11px] text-[#8b95a8]">
        Updated {ago(data.as_of)} · {data.data_state}
      </div>
      <p className="text-[11px] text-[#8b95a8] mt-3">
        Observable quality only. Not a price forecast. EXECUTE off.{" "}
        <a className="text-[#3ee58a] underline" href={`https://memecoin-os.web.app/twin/${data.token.id}`} target="_blank" rel="noreferrer">
          Open Twin
        </a>
      </p>
      <div className="text-[10px] text-[#5a6478] mt-3 tracking-wide">Powered by MemeCoin OS</div>
    </div>
  );
}
