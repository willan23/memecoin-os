"use client";

import { api } from "@/lib/api";
import type { Overview } from "@/lib/types";
import { DataStateBadge, ErrorState } from "@/components/ui";
import { scoreTone } from "@/lib/format";
import Link from "next/link";
import { useEffect, useState } from "react";

export default function ProjectsPage() {
  const [data, setData] = useState<Overview | null>(null);
  const [claims, setClaims] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.overview(), api.projects().catch(() => ({ projects: [] }))])
      .then(([ov, p]) => {
        setData(ov);
        const map: Record<string, string> = {};
        for (const row of p.projects) map[row.id] = row.claim_status;
        setClaims(map);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;
  if (!data) return <div className="text-sm text-mute">Loading ecosystems…</div>;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">MemeCoin OS for Projects</div>
        <h1 className="text-3xl mt-1">Project intelligence</h1>
        <p className="text-sm text-mute mt-2 max-w-2xl">
          Operational view of a tracked ecosystem. Official links are registry or operator profile.
          PROJECT CLAIM is a label, not VERIFIED. VERIFIED ≠ SAFE. Paying never changes a score.
        </p>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3">
        {data.tokens.map((t) => {
          const claim = claims[t.token.id] ?? "unclaimed";
          return (
            <Link key={t.token.id} href={`/projects/${t.token.id}`} className="panel p-5 hover:border-phosphor/40">
              <div className="font-mono text-phosphor text-xs">{t.token.symbol}</div>
              <div className="text-lg">{t.token.name}</div>
              <div className="mt-2 flex gap-2 items-center">
                <DataStateBadge state={t.data_state} />
                <span className={`font-mono text-sm ${scoreTone(t.health)}`}>{t.health.toFixed(0)}</span>
              </div>
              <div className="text-xs text-mute mt-3">
                {claim === "claimed" ? "PROJECT CLAIM" : "UNCLAIMED"} · {t.token.verification} · {t.token.primary_chain}
              </div>
            </Link>
          );
        })}
      </div>
    </div>
  );
}
