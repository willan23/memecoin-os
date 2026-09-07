"use client";

import { api } from "@/lib/api";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useState } from "react";

export default function BriefingPage() {
  const [items, setItems] = useState<
    {
      ecosystem_id: string;
      name: string;
      lifecycle: string;
      headline: string;
      changes: string[];
      risks: string[];
      health: number;
      confidence: number;
      data_state: string;
    }[]
  >([]);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .briefing()
      .then((d) => {
        setItems(d.items ?? []);
        setNote(d.note ?? null);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <div className="kicker">Daily intelligence</div>
        <h1 className="text-3xl mt-1">Ecosystem brief</h1>
        <p className="text-sm text-mute mt-2">
          Headlines from Twin snapshots. EXECUTE off. A blank source is not a quiet market.
        </p>
      </div>
      {note ? <p className="text-xs text-mute">{note}</p> : null}
      {items.map((it) => (
        <Link key={it.ecosystem_id} href={`/twin/${it.ecosystem_id}`} className="panel p-4 block hover:border-phosphor/40">
          <div className="flex justify-between gap-2">
            <div className="text-lg">{it.name}</div>
            <DataStateBadge state={it.data_state} />
          </div>
          <div className="font-mono text-xs text-mute mt-1">
            {it.lifecycle} · health {it.health.toFixed(0)} · conf {it.confidence.toFixed(2)}
          </div>
          <p className="text-sm mt-2">{it.headline || "No headline — evidence may be thin."}</p>
        </Link>
      ))}
    </div>
  );
}
