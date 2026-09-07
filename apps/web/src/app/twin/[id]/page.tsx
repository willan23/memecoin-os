"use client";

import { api, type TwinState } from "@/lib/api";
import { TwinGraphView } from "@/components/twin-graph";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEntityId } from "@/lib/route-id";
import { useEffect, useState } from "react";

const FIELDS: { key: keyof TwinState; label: string }[] = [
  { key: "market_state", label: "Market" },
  { key: "liquidity_state", label: "Liquidity" },
  { key: "holder_state", label: "Holders" },
  { key: "wallet_state", label: "Wallets" },
  { key: "whale_state", label: "Whales" },
  { key: "developer_state", label: "Development" },
  { key: "social_state", label: "Social" },
  { key: "narrative_state", label: "Narrative" },
  { key: "risk_state", label: "Risk" },
  { key: "governance_state", label: "Governance" },
];

export default function TwinDetailPage() {
  const id = useEntityId();
  const [twin, setTwin] = useState<TwinState | null>(null);
  const [graph, setGraph] = useState<Awaited<ReturnType<typeof api.twinGraph>> | null>(null);
  const [changed, setChanged] = useState<{ field: string; before: string; after: string; why: string }[]>([]);
  const [evidence, setEvidence] = useState<Awaited<ReturnType<typeof api.twinEvidence>> | null>(null);
  const [similar, setSimilar] = useState<{ b: string; similarity: number }[]>([]);
  const [phase, setPhase] = useState<string>("");
  const [ago, setAgo] = useState<string>("");
  const [view, setView] = useState<"2d" | "3d" | "list">("2d");
  const [frames, setFrames] = useState<{ t: string; kind: string; title: string }[]>([]);
  const [frame, setFrame] = useState(0);
  const [whale, setWhale] = useState<string>("");
  const [q, setQ] = useState("");
  const [copilotOut, setCopilotOut] = useState<string | null>(null);
  const [copilotBusy, setCopilotBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    Promise.all([
      ago ? api.twinState(id, ago) : api.twin(id),
      api.twinGraph(id),
      api.twinEvolution(id),
      api.twinEvidence(id),
      api.twinSimilar(id),
      api.twinLifecycle(id),
      api.twinReplay(id, "24h"),
      api.whaleBehaviour(id),
    ])
      .then(([t, g, evo, ev, sim, life, rep, wh]) => {
        setTwin(t.twin);
        setGraph(g);
        setChanged(evo.what_changed ?? []);
        setEvidence(ev);
        setSimilar((sim.similar ?? []).slice(0, 5).map((s) => ({ b: s.b, similarity: s.similarity })));
        setPhase(life.phase);
        setFrames(rep.frames ?? []);
        setWhale(wh.behaviour?.pattern ?? "");
      })
      .catch((e) => setError(String((e as Error).message ?? e)));
  }, [id, ago]);

  if (error) return <ErrorState message={error} />;
  if (!twin || !graph) return <div className="text-sm text-mute">Loading Twin…</div>;

  const highlight = frames[frame]?.kind;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap justify-between gap-3">
        <div>
          <div className="kicker">Digital Twin</div>
          <h1 className="text-3xl mt-1">{twin.ecosystem_id}</h1>
          <p className="text-sm text-mute mt-2">
            Lifecycle <span className="font-mono text-phosphor">{phase}</span> · whales{" "}
            <span className="font-mono">{whale || "unknown"}</span> · confidence {twin.confidence.toFixed(2)}{" "}
            · <DataStateBadge state={twin.freshness} />
          </p>
        </div>
        <div className="flex gap-2 items-start">
          {["", "24h", "7d", "30d"].map((w) => (
            <button
              key={w || "now"}
              className={`text-xs font-mono border rounded px-2 py-1 ${
                ago === w ? "border-phosphor text-phosphor" : "border-line text-mute"
              }`}
              onClick={() => setAgo(w)}
            >
              {w || "now"}
            </button>
          ))}
        </div>
      </div>

      <section className="panel p-4 overflow-auto">
        <div className="flex flex-wrap gap-2 justify-between mb-3">
          <div className="kicker">Twin graph</div>
          <div className="flex gap-1">
            {(["2d", "3d", "list"] as const).map((m) => (
              <button
                key={m}
                className={`text-xs font-mono border rounded px-2 py-1 uppercase ${
                  view === m ? "border-phosphor text-phosphor" : "border-line text-mute"
                }`}
                onClick={() => setView(m)}
              >
                {m}
              </button>
            ))}
          </div>
        </div>
        {view === "list" ? (
          <ul className="text-sm font-mono space-y-1">
            {graph.entities.map((n) => (
              <li key={n.id}>
                {n.kind} · {n.label}
              </li>
            ))}
          </ul>
        ) : (
          <TwinGraphView
            entities={graph.entities}
            relationships={graph.relationships}
            mode={view}
            highlightKind={highlight}
          />
        )}
        <p className="text-xs text-mute mt-2">{graph.note} 3D is a projection of the same model — no extra facts.</p>
      </section>

      <section className="panel p-4 space-y-2">
        <div className="kicker">Replay (observed frames only)</div>
        {frames.length === 0 ? (
          <p className="text-sm text-mute">No timeline/history frames in range. Not zero activity.</p>
        ) : (
          <>
            <input
              type="range"
              min={0}
              max={frames.length - 1}
              value={frame}
              onChange={(e) => setFrame(Number(e.target.value))}
              className="w-full"
            />
            <div className="text-sm font-mono">
              {frames[frame]?.t} · {frames[frame]?.kind} · {frames[frame]?.title}
            </div>
          </>
        )}
      </section>

      <section className="grid md:grid-cols-2 gap-3">
        {FIELDS.map(({ key, label }) => {
          const f = twin[key];
          if (typeof f !== "object" || !f || !("data_state" in f)) return null;
          return (
            <div key={key} className="panel p-4">
              <div className="flex justify-between">
                <div className="kicker">{label}</div>
                <DataStateBadge state={f.data_state} />
              </div>
              <p className="text-sm mt-2 font-mono">{f.summary}</p>
            </div>
          );
        })}
      </section>

      <section className="panel p-4 space-y-2">
        <div className="kicker">What changed (24h reconstruct)</div>
        {changed.length === 0 ? (
          <p className="text-sm text-mute">No evidenced delta — or no history ≤ 24h (Postgres).</p>
        ) : (
          changed.map((c) => (
            <div key={c.field} className="text-sm border-b border-line/50 py-2">
              <span className="font-mono text-phosphor">{c.field}</span>
              <div className="text-mute text-xs mt-1">{c.before} → {c.after}</div>
              <div className="text-mute text-xs">{c.why}</div>
            </div>
          ))
        )}
      </section>

      <section className="panel p-4 space-y-2">
        <div className="kicker">Evidence / Investigate next</div>
        {evidence?.claims.map((c) => (
          <div key={c.claim} className="text-sm">
            {c.claim} <span className="font-mono text-xs text-mute">{c.confidence.toFixed(2)}</span>
          </div>
        ))}
        <ul className="text-sm list-disc pl-5 text-mute">
          {evidence?.investigate_next.map((s) => (
            <li key={s.title}>
              {s.title} — {s.reason}
            </li>
          ))}
        </ul>
      </section>

      <section className="panel p-4">
        <div className="kicker mb-2">Similar ecosystems</div>
        <p className="text-xs text-mute mb-2">Structural only. Not “this will pump like that”.</p>
        {similar.map((s) => (
          <Link key={s.b} href={`/twin/${s.b}`} className="block font-mono text-sm py-1 hover:text-phosphor">
            {s.b} · {(s.similarity * 100).toFixed(0)}%
          </Link>
        ))}
      </section>

      <section className="panel p-4 space-y-2">
        <div className="kicker">Copilot</div>
        <p className="text-xs text-mute">Reads Twin/snapshots only. EXECUTE off. Not a trade signal.</p>
        <form
          className="flex gap-2"
          onSubmit={async (e) => {
            e.preventDefault();
            if (!q.trim()) return;
            setCopilotBusy(true);
            try {
              const r = await api.copilot(q, id);
              const summary = r.research?.executive_summary ?? r.research?.markdown ?? JSON.stringify(r);
              setCopilotOut(typeof summary === "string" ? summary : JSON.stringify(summary));
            } catch (err) {
              setError(String((err as Error).message ?? err));
            } finally {
              setCopilotBusy(false);
            }
          }}
        >
          <input
            className="flex-1 bg-void border border-line rounded px-2 py-1 text-sm"
            value={q}
            onChange={(e) => setQ(e.target.value)}
            placeholder="Ask about this Twin…"
          />
          <button className="bg-phosphor text-void text-sm rounded px-3 py-1" disabled={copilotBusy}>
            {copilotBusy ? "…" : "Ask"}
          </button>
        </form>
        {copilotOut ? <p className="text-sm whitespace-pre-wrap">{copilotOut}</p> : null}
      </section>

      <div className="flex gap-3 text-sm">
        <Link className="text-phosphor underline" href={`/simulations?id=${id}`}>
          Simulate
        </Link>
        <Link className="text-phosphor underline" href={`/twin/compare?a=${id}`}>
          Compare Twin
        </Link>
        <Link className="text-mute underline" href={`/tokens/${id}`}>
          Token snapshot
        </Link>
      </div>
      <p className="text-xs text-mute">{twin.note}</p>
    </div>
  );
}
