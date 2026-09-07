"use client";

import { api, eventsUrl } from "@/lib/api";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useState } from "react";

export default function TwinIndexPage() {
  const [rows, setRows] = useState<
    {
      id: string;
      name: string;
      lifecycle_phase: string;
      discovery_score: number;
      intelligence_score: number;
      risk_score: number;
      confidence: number;
      verification_status: string;
      primary_chain: string;
      data_freshness: string;
    }[]
  >([]);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [live, setLive] = useState("connecting");

  useEffect(() => {
    const load = () =>
      api
        .ecosystems()
        .then((d) => {
          setRows(d.ecosystems ?? []);
          setNote(d.note ?? null);
        })
        .catch((e) => setError(String(e.message ?? e)));
    load();
    const es = new EventSource(eventsUrl());
    es.onopen = () => setLive("sse");
    es.onmessage = () => {
      setLive("twin.updated");
      load();
    };
    es.onerror = () => setLive("sse-degraded");
    return () => es.close();
  }, []);

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Digital Twin</div>
        <h1 className="text-3xl mt-1">Ecosystems</h1>
        <p className="text-sm text-mute mt-2">
          Live state model composed from snapshots. Not a price tracker. VERIFIED ≠ SAFE. auto_verified is always false.
          Stream: <span className="font-mono">{live}</span>
        </p>
      </div>
      <p className="text-sm">
        <Link href="/twin/compare" className="text-phosphor underline">
          Compare two Twins
        </Link>
      </p>
      {note ? <p className="text-xs text-mute">{note}</p> : null}
      <div className="grid gap-3 md:grid-cols-2">
        {rows.map((e) => (
          <Link key={e.id} href={`/twin/${e.id}`} className="panel p-4 hover:border-phosphor/40">
            <div className="flex justify-between gap-2">
              <div className="text-lg">{e.name}</div>
              <DataStateBadge state={e.data_freshness} />
            </div>
            <div className="mt-2 font-mono text-xs text-mute">
              {e.lifecycle_phase} · {e.primary_chain} · {e.verification_status}
            </div>
            <div className="mt-3 grid grid-cols-3 gap-2 text-xs font-mono">
              <div>intel {e.intelligence_score.toFixed(0)}</div>
              <div>risk {e.risk_score.toFixed(0)}</div>
              <div>disc {e.discovery_score.toFixed(0)}</div>
            </div>
          </Link>
        ))}
      </div>
    </div>
  );
}
