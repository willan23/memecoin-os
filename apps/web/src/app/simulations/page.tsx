"use client";

import { api } from "@/lib/api";
import { ErrorState } from "@/components/ui";
import { useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";

function SimulationsInner() {
  const params = useSearchParams();
  const [ecosystemId, setEcosystemId] = useState(params.get("id") ?? "");
  const [ids, setIds] = useState<string[]>([]);
  const [liq, setLiq] = useState(25);
  const [holders, setHolders] = useState(0);
  const [whale, setWhale] = useState(0);
  const [dev, setDev] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<Awaited<ReturnType<typeof api.simulate>>["result"] | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    api
      .ecosystems()
      .then((d) => setIds(d.ecosystems.map((e) => e.id)))
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6 max-w-xl">
      <div>
        <div className="kicker">What-if</div>
        <h1 className="text-3xl mt-1">Simulation</h1>
        <p className="text-sm text-mute mt-2">
          Clone Twin metrics and apply named shocks. Not a prediction. Price is not simulated. EXECUTE stays off.
        </p>
      </div>
      <form
        className="panel p-5 space-y-4"
        onSubmit={async (e) => {
          e.preventDefault();
          setBusy(true);
          try {
            const r = await api.simulate({
              ecosystem_id: ecosystemId,
              liquidity_pct: liq,
              holder_growth_pct: holders,
              whale_selling_pct: whale,
              developer_pct: dev,
            });
            setResult(r.result);
          } catch (err) {
            setError(String((err as Error).message));
          } finally {
            setBusy(false);
          }
        }}
      >
        <label className="block text-sm">
          Ecosystem
          <select
            className="mt-1 w-full bg-void border border-line rounded px-3 py-2"
            value={ecosystemId}
            onChange={(e) => setEcosystemId(e.target.value)}
          >
            <option value="">Select</option>
            {ids.map((id) => (
              <option key={id} value={id}>
                {id}
              </option>
            ))}
          </select>
        </label>
        {[
          ["Liquidity %", liq, setLiq],
          ["Holder growth %", holders, setHolders],
          ["Whale selling %", whale, setWhale],
          ["Developer %", dev, setDev],
        ].map(([label, val, set]) => (
          <label key={String(label)} className="block text-sm">
            {label as string}
            <input
              type="number"
              className="mt-1 w-full bg-void border border-line rounded px-3 py-2 font-mono"
              value={val as number}
              onChange={(e) => (set as (n: number) => void)(Number(e.target.value))}
            />
          </label>
        ))}
        <button
          disabled={busy || !ecosystemId}
          className="bg-phosphor text-void text-sm rounded px-3 py-2 disabled:opacity-50"
        >
          {busy ? "Running…" : "Run simulation"}
        </button>
      </form>
      {result ? (
        <section className="panel p-5 space-y-2">
          <div className="font-mono text-phosphor text-sm">{result.banner}</div>
          <div className="text-sm">
            Health {result.current_health.toFixed(1)} → {result.projected_health.toFixed(1)} (Δ{" "}
            {result.delta_health.toFixed(1)})
          </div>
          <div className="text-sm">
            Risk {result.current_risk.toFixed(1)} → {result.projected_risk.toFixed(1)}
          </div>
          <div className="text-xs text-mute">confidence {result.confidence.toFixed(2)}</div>
          <ul className="text-xs text-mute list-disc pl-5">
            {result.notes.map((n) => (
              <li key={n}>{n}</li>
            ))}
          </ul>
          <div className="text-xs font-mono">
            {result.sensitivity.map((s) => (
              <div key={s.variable}>
                {s.variable}: {s.impact}
              </div>
            ))}
          </div>
        </section>
      ) : null}
    </div>
  );
}

export default function SimulationsPage() {
  return (
    <Suspense fallback={<div className="text-sm text-mute">Loading…</div>}>
      <SimulationsInner />
    </Suspense>
  );
}
