"use client";

import { api } from "@/lib/api";
import { ErrorState } from "@/components/ui";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";

function Inner() {
  const q = useSearchParams();
  const [ids, setIds] = useState<string[]>([]);
  const [a, setA] = useState(q.get("a") ?? "");
  const [b, setB] = useState(q.get("b") ?? "");
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<Awaited<ReturnType<typeof api.compareTwins>> | null>(null);

  useEffect(() => {
    api
      .ecosystems()
      .then((d) => {
        const list = d.ecosystems.map((e) => e.id);
        setIds(list);
        if (!a && list[0]) setA(list[0]);
        if (!b && list[1]) setB(list[1]);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6 max-w-2xl">
      <div>
        <div className="kicker">Twin compare</div>
        <h1 className="text-3xl mt-1">State A vs State B</h1>
        <p className="text-sm text-mute mt-2">Structural Twin compare. Not “this will repeat that price path”.</p>
      </div>
      <form
        className="panel p-4 flex flex-wrap gap-2"
        onSubmit={async (e) => {
          e.preventDefault();
          try {
            setResult(await api.compareTwins(a, b));
          } catch (err) {
            setError(String((err as Error).message));
          }
        }}
      >
        <select className="bg-void border border-line rounded px-2 py-1" value={a} onChange={(e) => setA(e.target.value)}>
          {ids.map((id) => (
            <option key={id}>{id}</option>
          ))}
        </select>
        <select className="bg-void border border-line rounded px-2 py-1" value={b} onChange={(e) => setB(e.target.value)}>
          {ids.map((id) => (
            <option key={id}>{id}</option>
          ))}
        </select>
        <button className="bg-phosphor text-void text-sm rounded px-3 py-1">Compare</button>
      </form>
      {result ? (
        <section className="panel p-4 space-y-2">
          <div className="font-mono text-phosphor">
            similarity {(result.similarity.similarity * 100).toFixed(0)}%
          </div>
          <p className="text-xs text-mute">{result.similarity.disclaimer}</p>
          <div className="text-sm">
            {result.a.ecosystem.name} ({result.a.ecosystem.lifecycle_phase}) vs {result.b.ecosystem.name} (
            {result.b.ecosystem.lifecycle_phase})
          </div>
        </section>
      ) : null}
      <Link href="/twin" className="text-sm text-phosphor underline">
        Back to Twin list
      </Link>
    </div>
  );
}

export default function TwinComparePage() {
  return (
    <Suspense fallback={<div className="text-sm text-mute">Loading…</div>}>
      <Inner />
    </Suspense>
  );
}
