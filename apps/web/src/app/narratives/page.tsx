"use client";

import { api } from "@/lib/api";
import { ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useState } from "react";

type Narrative = {
  id: string;
  label: string;
  state: string;
  confidence: number;
  token_ids: string[];
  mention_velocity: string;
  unique_accounts: string;
  evidence?: string[];
};

type GCluster = {
  cluster_id: string;
  label: string;
  token_ids: string[];
  confidence: number;
};

export default function NarrativesPage() {
  const [narratives, setNarratives] = useState<Narrative[]>([]);
  const [clusters, setClusters] = useState<GCluster[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.narratives(), api.genomeClusters()])
      .then(([n, g]) => {
        setNarratives(n.narratives ?? []);
        setClusters(g.clusters ?? []);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Phase 3</div>
        <h1 className="text-3xl mt-1">Narrative Radar</h1>
        <p className="text-sm text-mute mt-2">
          Tags from the registry plus weak name heuristics. Mention velocity stays not connected without a firehose. Genome clusters do not use token names.
        </p>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {narratives.length === 0 ? (
          <div className="text-sm text-mute col-span-2">No narratives persisted. Refresh with Postgres.</div>
        ) : (
          narratives.map((n) => (
            <div key={n.id} className="panel p-4">
              <div className="flex justify-between items-baseline">
                <div className="font-medium">{n.label}</div>
                <div className="kicker">{n.state}</div>
              </div>
              <div className="text-xs text-mute mt-2">
                conf {(n.confidence * 100).toFixed(0)}% · mentions {n.mention_velocity} · accounts {n.unique_accounts}
              </div>
              <div className="mt-2 flex flex-wrap gap-1">
                {n.token_ids.map((id) => (
                  <Link key={id} href={`/tokens/${id}`} className="text-xs border border-line rounded px-2 py-0.5 text-mute">
                    {id}
                  </Link>
                ))}
              </div>
            </div>
          ))
        )}
      </div>
      <div>
        <div className="kicker mb-2">Genome clusters</div>
        {clusters.length === 0 ? (
          <div className="text-sm text-mute">Need ≥2 similar genomes after a refresh.</div>
        ) : (
          <ul className="space-y-2">
            {clusters.map((c) => (
              <li key={c.cluster_id} className="panel p-3 text-sm">
                <span className="font-medium">{c.label}</span>
                <span className="text-mute"> · {(c.confidence * 100).toFixed(0)}% · {c.token_ids.join(", ")}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
    </div>
  );
}
