"use client";

import { api } from "@/lib/api";
import { compactUsd } from "@/lib/format";
import { operatorKey } from "@/lib/operator";
import { useEntityId } from "@/lib/route-id";
import type { TokenSnapshot } from "@/lib/types";
import { DataStateBadge, ErrorState, Metric } from "@/components/ui";
import Link from "next/link";
import { useEffect, useState } from "react";

type Official = {
  website: string;
  twitter: string;
  telegram: string;
  discord: string;
  github: string;
  docs: string;
  roadmap: string;
};

const EMPTY: Official = {
  website: "",
  twitter: "",
  telegram: "",
  discord: "",
  github: "",
  docs: "",
  roadmap: "",
};

function missingOr(v?: string) {
  if (!v || v === "MISSING") return "";
  return v;
}

export default function ProjectDeskPage() {
  const id = useEntityId();
  const [data, setData] = useState<TokenSnapshot | null>(null);
  const [claimStatus, setClaimStatus] = useState("unclaimed");
  const [claimLabel, setClaimLabel] = useState("");
  const [computed, setComputed] = useState<string | null>(null);
  const [official, setOfficial] = useState<Official>(EMPTY);
  const [form, setForm] = useState<Official>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [alerts, setAlerts] = useState<
    { id: string; kind: string; severity: string; title: string; body: string; evidence: string[]; fired_at: string }[]
  >([]);
  const [catalog, setCatalog] = useState<{ kind: string; label: string; availability: string; reason: string }[]>([]);
  const [hookKind, setHookKind] = useState("discord");
  const [hookUrl, setHookUrl] = useState("");
  const [hookChat, setHookChat] = useState("");
  const canWrite = Boolean(operatorKey());

  async function reload() {
    if (!id) return;
    const [snap, proj, al] = await Promise.all([
      api.token(id),
      api.project(id),
      api.projectAlerts(id).catch(() => ({ alerts: [], catalog: [] })),
    ]);
    setData(snap);
    setClaimStatus(proj.claim.status);
    setComputed(proj.verification.computed);
    setOfficial(proj.official);
    setForm({
      website: missingOr(proj.official.website),
      twitter: missingOr(proj.official.twitter),
      telegram: missingOr(proj.official.telegram),
      discord: missingOr(proj.official.discord),
      github: missingOr(proj.official.github),
      docs: missingOr(proj.official.docs),
      roadmap: missingOr(proj.official.roadmap),
    });
    setAlerts(al.alerts ?? []);
    setCatalog(al.catalog ?? []);
  }

  useEffect(() => {
    if (!id) return;
    setData(null);
    reload().catch((e) => setError(String((e as Error).message ?? e)));
  }, [id]);

  if (error) return <ErrorState message={error} />;
  if (!id || !data) return <div className="text-sm text-mute">Loading project desk…</div>;

  async function onClaim() {
    setBusy(true);
    setNote(null);
    try {
      await api.claimProject(id, claimLabel || undefined);
      await reload();
      setNote("Claim recorded. Verification and scores unchanged.");
    } catch (e) {
      setNote(String((e as Error).message ?? e));
    } finally {
      setBusy(false);
    }
  }

  async function onUnclaim() {
    setBusy(true);
    setNote(null);
    try {
      await api.unclaimProject(id);
      await reload();
      setNote("Claim withdrawn. Official links stay. Scores unchanged.");
    } catch (e) {
      setNote(String((e as Error).message ?? e));
    } finally {
      setBusy(false);
    }
  }

  async function onSaveLinks() {
    setBusy(true);
    setNote(null);
    try {
      await api.patchProjectProfile(id, form);
      await reload();
      setNote("Official links saved. Scores and verification unchanged.");
    } catch (e) {
      setNote(String((e as Error).message ?? e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Project intelligence dashboard</div>
        <h1 className="text-3xl mt-1">
          {data.token.name} <span className="text-mute font-mono text-xl">{data.token.symbol}</span>
        </h1>
        <p className="text-sm text-mute mt-2">
          Declared {data.token.verification}
          {computed ? ` · computed ${computed}` : ""}.{" "}
          {claimStatus === "claimed" ? "PROJECT CLAIM" : "UNCLAIMED"}. VERIFIED ≠ SAFE. Paying never
          changes a score.
        </p>
        <div className="mt-2">
          <DataStateBadge state={data.data_state} />
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
        <Metric label="Ecosystem health" value={data.scores.value.toFixed(0)} />
        <Metric label="Risk" value={`${data.risk.score.toFixed(0)} ${data.risk.level}`} />
        <Metric label="Liquidity" value={compactUsd(data.liquidity.liquidity_usd)} hint={data.liquidity.data_state} />
        <Metric
          label="Holders"
          value={data.onchain.data_state === "live" || data.onchain.data_state === "recent" ? String(data.onchain.holders) : "—"}
          hint={data.onchain.data_state}
        />
      </div>

      <div className="grid md:grid-cols-2 gap-3">
        <div className="panel p-4 space-y-2 text-sm">
          <div className="kicker">Official links</div>
          <LinkRow label="Website" value={official.website} />
          <LinkRow label="X / Twitter" value={official.twitter} />
          <LinkRow label="Telegram" value={official.telegram} />
          <LinkRow label="GitHub" value={official.github} />
          <LinkRow label="Discord" value={official.discord} />
          <LinkRow label="Docs" value={official.docs} />
          <LinkRow label="Roadmap" value={official.roadmap} />
          <p className="text-xs text-mute">
            Chat da plataforma está LIVE. Menções de firehose ficam não ligadas sem licença.
          </p>
          <Link className="text-phosphor text-sm" href={`/community?room=${encodeURIComponent(id)}`}>
            Abrir chat deste projeto
          </Link>
        </div>
        <div className="panel p-4 space-y-2 text-sm">
          <div className="kicker">Product loop</div>
          <div className="flex flex-wrap gap-2">
            <Link className="text-phosphor" href={`/tokens/${data.token.id}`}>Token</Link>
            <Link className="text-phosphor" href={`/twin/${data.token.id}`}>Digital Twin</Link>
            <Link className="text-phosphor" href="/discovery">Discovery</Link>
            <Link className="text-phosphor" href="/narratives">Narrative</Link>
            <Link className="text-phosphor" href="/simulations">Simulate</Link>
            <Link className="text-phosphor" href="/alerts">Alerts</Link>
            <Link className="text-phosphor" href={`/intelligence?token=${data.token.id}`}>AI analysis</Link>
          </div>
          <p className="text-xs text-mute">
            Project alerts reuse the operator webhook desk. No invented community-growth numbers.
          </p>
        </div>
      </div>

      <div className="panel p-5 space-y-3">
        <div className="kicker">Project alerts</div>
        <p className="text-xs text-mute">
          Detection only. Not a buy signal. Kinds without a live source stay not connected — not a quiet market.
        </p>
        <div className="flex flex-wrap gap-2">
          {catalog.map((c) => (
            <span
              key={c.kind}
              title={c.reason}
              className={`font-mono text-[11px] border rounded px-2 py-1 ${
                c.availability === "live" ? "border-phosphor text-phosphor" : "border-line text-mute"
              }`}
            >
              {c.label} · {c.availability}
            </span>
          ))}
        </div>
        {alerts.length === 0 ? (
          <div className="text-sm text-mute">No persisted alerts for this ecosystem.</div>
        ) : (
          <ul className="space-y-2">
            {alerts.map((a) => (
              <li key={a.id} className="border border-line rounded-lg p-3">
                <div className="font-mono text-[11px] text-mute uppercase">
                  {a.severity} · {a.kind}
                </div>
                <div className="text-sm mt-1">{a.title}</div>
                <p className="text-xs text-mute mt-1">{a.body}</p>
              </li>
            ))}
          </ul>
        )}
        <div className="flex flex-wrap gap-2 text-sm">
          <a className="text-phosphor underline" href={`/v1/connect/discord?token_id=${encodeURIComponent(id)}`}>
            Connect Discord (OAuth)
          </a>
          <button
            type="button"
            className="text-phosphor underline"
            onClick={async () => {
              try {
                const t = await api.telegramStart(id);
                window.open(t.url, "_blank", "noopener,noreferrer");
              } catch (e) {
                setNote(String((e as Error).message ?? e));
              }
            }}
          >
            Connect Telegram (bot)
          </button>
          <Link className="text-phosphor underline" href="/community">
            Open community chat
          </Link>
        </div>
        {canWrite ? (
          <form
            className="grid gap-2 md:grid-cols-[120px_1fr_1fr_auto]"
            onSubmit={async (e) => {
              e.preventDefault();
              setBusy(true);
              setNote(null);
              try {
                await api.createWebhook({
                  kind: hookKind,
                  url: hookUrl,
                  chat_id: hookChat || undefined,
                  token_id: id,
                });
                setHookUrl("");
                setHookChat("");
                setNote("Token-scoped webhook saved. Only this project’s alerts are delivered.");
                await reload();
              } catch (err) {
                setNote(String((err as Error).message ?? err));
              } finally {
                setBusy(false);
              }
            }}
          >
            <select
              className="bg-void border border-line rounded px-2 py-2 text-sm"
              value={hookKind}
              onChange={(e) => setHookKind(e.target.value)}
            >
              <option value="discord">discord</option>
              <option value="telegram">telegram</option>
            </select>
            <input
              className="bg-void border border-line rounded px-3 py-2 text-sm font-mono"
              placeholder="https://discord.com/api/webhooks/…"
              value={hookUrl}
              onChange={(e) => setHookUrl(e.target.value)}
              required
            />
            <input
              className="bg-void border border-line rounded px-3 py-2 text-sm font-mono"
              placeholder="Telegram chat_id"
              value={hookChat}
              onChange={(e) => setHookChat(e.target.value)}
            />
            <button className="bg-phosphor text-void text-sm rounded px-3 py-2" type="submit" disabled={busy}>
              Bind webhook
            </button>
          </form>
        ) : (
          <p className="text-xs text-mute">
            Operator key in Settings to bind a Discord/Telegram webhook to this project.
          </p>
        )}
      </div>

      <div className="panel p-5 space-y-3">
        <div className="kicker">Claim and official profile</div>
        <p className="text-xs text-mute">
          Operator write key required (Settings). Claim does not verify identity, contract, or data.
          Links must be https. This never upgrades health or risk.
        </p>
        {!canWrite ? (
          <p className="text-sm">
            Save the operator key in <Link className="text-phosphor" href="/settings">Settings</Link> to
            claim or edit links.
          </p>
        ) : (
          <>
            <div className="flex flex-col sm:flex-row gap-2">
              <input
                className="flex-1 bg-void border border-line rounded-lg px-3 py-2 text-sm"
                placeholder="Optional claimant label"
                value={claimLabel}
                onChange={(e) => setClaimLabel(e.target.value)}
              />
              {claimStatus === "claimed" ? (
                <button className="bg-void border border-line rounded-lg px-4 py-2 text-sm" disabled={busy} onClick={onUnclaim}>
                  Withdraw claim
                </button>
              ) : (
                <button className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm" disabled={busy} onClick={onClaim}>
                  Claim project
                </button>
              )}
            </div>
            <div className="grid md:grid-cols-2 gap-2">
              {(
                [
                  ["website", "Website"],
                  ["twitter", "X / Twitter"],
                  ["telegram", "Telegram"],
                  ["discord", "Discord"],
                  ["github", "GitHub"],
                  ["docs", "Docs"],
                  ["roadmap", "Roadmap"],
                ] as const
              ).map(([key, label]) => (
                <label key={key} className="text-xs text-mute">
                  {label}
                  <input
                    className="mt-1 w-full bg-void border border-line rounded-lg px-3 py-2 text-sm font-mono text-ink"
                    placeholder="https://"
                    value={form[key]}
                    onChange={(e) => setForm({ ...form, [key]: e.target.value })}
                  />
                </label>
              ))}
            </div>
            <button className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm" disabled={busy} onClick={onSaveLinks}>
              Save official links
            </button>
          </>
        )}
        {note ? <p className="text-xs text-mute">{note}</p> : null}
      </div>
    </div>
  );
}

function LinkRow({ label, value }: { label: string; value: string }) {
  if (!value || value === "MISSING") {
    return <div className="text-mute">{label}: —</div>;
  }
  return (
    <div>
      {label}:{" "}
      <a className="text-phosphor break-all" href={value} target="_blank" rel="noreferrer">
        {value}
      </a>
    </div>
  );
}
