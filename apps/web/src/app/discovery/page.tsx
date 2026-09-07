"use client";

import { api } from "@/lib/api";
import { operatorKey } from "@/lib/operator";
import { ErrorState } from "@/components/ui";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState } from "react";

type Candidate = {
  chain_id: string;
  address: string;
  symbol?: string | null;
  name?: string | null;
  status: string;
  score: number;
  auto_verified: boolean;
  already_tracked?: boolean;
  liquidity_usd?: number | null;
  evidence?: string[];
};

export default function DiscoveryPage() {
  const router = useRouter();
  const [rows, setRows] = useState<Candidate[]>([]);
  const [disclaimer, setDisclaimer] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [rowError, setRowError] = useState<string | null>(null);
  const [canWrite, setCanWrite] = useState(false);

  useEffect(() => {
    api
      .discovery()
      .then((d) => {
        setRows(d.candidates ?? []);
        setDisclaimer(d.disclaimer ?? null);
      })
      .catch((e) => setError(String(e.message ?? e)));
    setCanWrite(Boolean(operatorKey()));
  }, []);

  async function track(r: Candidate) {
    const key = `${r.chain_id}-${r.address}`;
    setBusy(key);
    setRowError(null);
    try {
      const symbol = (r.symbol ?? "").trim() || r.address.slice(2, 6).toUpperCase();
      const name = (r.name ?? "").trim() || symbol;
      const snap = await api.onboard({
        name,
        symbol,
        chain: r.chain_id,
        address: r.address,
      });
      setRows((prev) =>
        prev.map((row) =>
          row.address.toLowerCase() === r.address.toLowerCase()
            ? { ...row, already_tracked: true }
            : row,
        ),
      );
      router.push(`/tokens/${snap.token.id}`);
    } catch (e) {
      setRowError(String((e as Error).message));
    } finally {
      setBusy(null);
    }
  }

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Phase 3</div>
        <h1 className="text-3xl mt-1">Discovery</h1>
        <p className="text-sm text-mute mt-2">
          Counterparts from DexScreener pairs of tracked tokens. Track writes UNVERIFIED only. Growth never assigns VERIFIED. VERIFIED ≠ SAFE.
          {!canWrite ? (
            <>
              {" "}
              <Link href="/settings" className="text-signal">
                Set the operator key in Settings
              </Link>{" "}
              to Track.
            </>
          ) : null}
        </p>
      </div>
      {disclaimer ? <div className="text-xs text-mute">{disclaimer}</div> : null}
      {rowError ? <div className="text-sm text-danger">{rowError}</div> : null}
      {rows.length === 0 ? (
        <div className="text-sm text-mute">No counterparts persisted yet. Refresh tokens with DexScreener + Postgres.</div>
      ) : (
        <div className="panel overflow-auto">
          <table className="w-full text-sm">
            <thead className="text-mute font-mono text-[11px] uppercase border-b border-line">
              <tr>
                {["Symbol", "Chain", "Status", "Score", "Verified?", "Address", ""].map((h) => (
                  <th key={h || "action"} className="text-left px-3 py-2 font-medium">{h}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => {
                const key = `${r.chain_id}-${r.address}`;
                return (
                  <tr key={key} className="border-b border-line/60">
                    <td className="px-3 py-2">{r.symbol ?? "—"}</td>
                    <td className="px-3 py-2 font-mono">{r.chain_id}</td>
                    <td className="px-3 py-2">{r.status}</td>
                    <td className="px-3 py-2 font-mono">{r.score.toFixed(2)}</td>
                    <td className="px-3 py-2">{r.auto_verified ? "yes" : "no"}</td>
                    <td className="px-3 py-2 font-mono text-xs">{r.address.slice(0, 10)}…{r.address.slice(-4)}</td>
                    <td className="px-3 py-2 text-right">
                      {r.already_tracked ? (
                        <span className="text-xs text-mute">Tracked</span>
                      ) : (
                        <button
                          className="bg-phosphor text-void rounded-lg px-3 py-1 text-xs font-medium disabled:opacity-50"
                          disabled={busy === key || !canWrite}
                          onClick={() => track(r)}
                        >
                          {busy === key ? "Indexing…" : "Track"}
                        </button>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
