"use client";

import { ErrorState } from "@/components/ui";
import { useEffect, useState } from "react";

type Health = {
  ok: boolean;
  degraded: boolean;
  data_mode: string;
  postgres?: { ok: boolean; latency_ms?: number | null };
  redis?: { ok: boolean; latency_ms?: number | null };
  providers?: Record<string, { last_ok_at?: string | null; last_status?: number | null; rate_limited?: number; latency_ms?: number | null; detail?: string | null }>;
  last_job?: Record<string, unknown> | null;
  tokens?: { live: number; missing: number; conflict: number; stale: number };
};

export default function StatusPage() {
  const [data, setData] = useState<Health | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetch("/health", { cache: "no-store" })
      .then(async (r) => {
        if (!r.ok) throw new Error(`health ${r.status}`);
        return r.json();
      })
      .then(setData)
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;
  if (!data) return <div className="text-mute text-sm">Reading operators…</div>;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Observability</div>
        <h1 className="text-3xl mt-1">Status</h1>
        <p className="text-sm text-mute mt-2">
          Live vs missing counts. Postgres/Redis latency. Job lag. No invented numbers.
        </p>
      </div>
      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Tile label="Mode" value={data.data_mode} warn={data.degraded} />
        <Tile label="Postgres" value={data.postgres?.ok ? `${data.postgres.latency_ms ?? "—"} ms` : "DOWN"} warn={!data.postgres?.ok} />
        <Tile label="Redis" value={data.redis?.ok ? `${data.redis.latency_ms ?? "—"} ms` : "DOWN"} warn={!data.redis?.ok} />
        <Tile label="Degraded" value={data.degraded ? "YES" : "no"} warn={data.degraded} />
      </div>
      <div className="panel p-4">
        <div className="kicker mb-3">Tokens by data_state</div>
        <div className="grid grid-cols-4 gap-3 font-mono text-sm">
          <div>LIVE {data.tokens?.live ?? 0}</div>
          <div>NOT CONNECTED {data.tokens?.missing ?? 0}</div>
          <div>CONFLICT {data.tokens?.conflict ?? 0}</div>
          <div>STALE {data.tokens?.stale ?? 0}</div>
        </div>
      </div>
      <div className="panel p-4">
        <div className="kicker mb-3">Last job</div>
        <pre className="text-xs text-mute overflow-auto">
          {JSON.stringify(data.last_job ?? { note: "no job_runs yet" }, null, 2)}
        </pre>
      </div>
      <div className="panel overflow-hidden">
        <div className="kicker px-4 pt-4 mb-2">Providers</div>
        <table className="w-full text-sm">
          <thead className="text-mute font-mono text-[11px] uppercase border-b border-line">
            <tr>
              {["Provider", "Last OK", "Status", "429s", "Latency", "Detail"].map((h) => (
                <th key={h} className="text-left font-medium px-4 py-2">{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Object.entries(data.providers ?? {}).map(([name, p]) => (
              <tr key={name} className="border-b border-line/60">
                <td className="px-4 py-2 font-mono">{name}</td>
                <td className="px-4 py-2 text-mute text-xs">{p.last_ok_at ?? "—"}</td>
                <td className="px-4 py-2 font-mono">{p.last_status ?? "—"}</td>
                <td className="px-4 py-2 font-mono">{p.rate_limited ?? 0}</td>
                <td className="px-4 py-2 font-mono">{p.latency_ms != null ? `${p.latency_ms} ms` : "—"}</td>
                <td className="px-4 py-2 text-mute text-xs">{p.detail ?? "—"}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {Object.keys(data.providers ?? {}).length === 0 ? (
          <div className="px-4 py-3 text-sm text-mute">No provider_health rows yet.</div>
        ) : null}
      </div>
    </div>
  );
}

function Tile({ label, value, warn }: { label: string; value: string; warn?: boolean }) {
  return (
    <div className="panel p-4">
      <div className="kicker">{label}</div>
      <div className={`mt-2 font-mono text-xl ${warn ? "text-danger" : "text-phosphor"}`}>{value}</div>
    </div>
  );
}
