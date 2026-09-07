"use client";

import { api } from "@/lib/api";
import type { AgentAnswer } from "@/lib/types";
import { ErrorState } from "@/components/ui";
import { useState } from "react";

const PROMPTS = [
  "Which memecoins are gaining organic momentum?",
  "Which ecosystems are losing activity?",
  "What are the biggest risks across tracked tokens?",
  "Which successful mechanisms appear across multiple ecosystems?",
];

export default function IntelligencePage() {
  const [question, setQuestion] = useState(PROMPTS[0]);
  const [tokenId, setTokenId] = useState("");
  const [result, setResult] = useState<AgentAnswer | null>(null);
  const [report, setReport] = useState<{
    markdown: string;
    executive_summary: string;
    model_id: string;
    confidence: number;
    grounded: boolean;
    policy: { execute: boolean };
    unknown: string[];
    sections: { heading: string; body: string; data_state: string }[];
  } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function run() {
    setBusy(true);
    setError(null);
    try {
      const [r, rep] = await Promise.all([
        api.ask(question, tokenId || undefined),
        api.research(question, tokenId || undefined),
      ]);
      setResult(r);
      setReport(rep);
    } catch (e) {
      setError(String((e as Error).message));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6 max-w-4xl">
      <div>
        <div className="kicker">MemeCoin Intelligence Agent</div>
        <h1 className="text-3xl mt-1">Research agent</h1>
        <p className="text-sm text-mute mt-2">
          Intent → evidence retrieval → specialist agents → grounded report. RAG is lexical over snapshots, not a substitute for Postgres. EXECUTE is disabled. No price targets.
        </p>
      </div>
      <div className="flex flex-wrap gap-2">
        {PROMPTS.map((p) => (
          <button
            key={p}
            onClick={() => setQuestion(p)}
            className="text-xs border border-line rounded-full px-3 py-1 text-mute hover:text-ink"
          >
            {p}
          </button>
        ))}
      </div>
      <div className="panel p-4 space-y-3">
        <input
          className="w-full bg-void border border-line rounded-lg px-3 py-2 text-sm"
          placeholder="Optional token id (from /tokens)"
          value={tokenId}
          onChange={(e) => setTokenId(e.target.value)}
        />
        <textarea
          className="w-full h-28 bg-void border border-line rounded-lg p-3 text-sm"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
        />
        <button
          disabled={busy}
          onClick={run}
          className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm font-medium disabled:opacity-50"
        >
          {busy ? "Reasoning…" : "Ask"}
        </button>
      </div>
      {error ? <ErrorState message={error} /> : null}
      {result ? (
        <div className="panel p-5 space-y-4">
          <div className="text-xs font-mono text-mute">
            {result.model} · grounded={String(result.grounded)} · conf{" "}
            {(result.confidence * 100).toFixed(0)}% · execute=
            {String(result.policy.execute)}
          </div>
          <pre className="whitespace-pre-wrap text-sm leading-relaxed">{result.answer}</pre>
          {result.observations.map((o) => (
            <div key={o.observation} className="border-t border-line pt-3 text-sm">
              <div>{o.observation}</div>
              <div className="text-mute text-xs mt-1">{o.interpretation}</div>
              <div className="text-[11px] text-mute mt-1">evidence: {o.evidence.join(" · ")}</div>
            </div>
          ))}
        </div>
      ) : null}
      {report ? (
        <div className="panel p-5 space-y-3">
          <div className="text-xs font-mono text-mute">
            {report.model_id} · grounded={String(report.grounded)} · conf {(report.confidence * 100).toFixed(0)}% · execute=
            {String(report.policy.execute)}
          </div>
          <div className="text-sm">{report.executive_summary}</div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
            {report.sections.map((s) => (
              <div key={s.heading} className="border border-line rounded p-3 text-xs">
                <div className="kicker">{s.heading} · {s.data_state}</div>
                <div className="mt-1">{s.body}</div>
              </div>
            ))}
          </div>
          {report.unknown.length > 0 ? (
            <div className="text-xs text-mute">Missing: {report.unknown.join(" · ")}</div>
          ) : null}
          {tokenId ? (
            <a className="text-xs font-mono text-signal" href={`/v1/tokens/${tokenId}/research.md`}>
              research.md
            </a>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
