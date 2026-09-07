"use client";

import { api } from "@/lib/api";
import { ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useState } from "react";

type Alert = {
  id: string;
  token_id: string;
  kind: string;
  severity: string;
  title: string;
  body: string;
  evidence: string[];
  fired_at: string;
};

export default function AlertsPage() {
  const [alerts, setAlerts] = useState<Alert[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [kind, setKind] = useState("discord");
  const [url, setUrl] = useState("");
  const [chatId, setChatId] = useState("");
  const [hooks, setHooks] = useState<{ id: string; kind: string; url: string }[]>([]);
  const [rules, setRules] = useState<{ kind: string; enabled: boolean }[]>([]);

  useEffect(() => {
    api
      .alerts()
      .then((r) => setAlerts(r.alerts))
      .catch((e) => setError(String(e.message ?? e)));
    api.webhooks().then((r) => setHooks(r.webhooks ?? [])).catch(() => setHooks([]));
    api.alertRules().then((r) => setRules(r.rules ?? [])).catch(() => setRules([]));
  }, []);

  if (error) return <ErrorState message={error} />;
  if (!alerts) return <div className="text-mute text-sm">Scanning snapshots…</div>;

  return (
    <div className="space-y-6">
      <div>
        <div className="kicker">Early warning</div>
        <h1 className="text-3xl mt-1">Alerts</h1>
        <p className="text-sm text-mute mt-2">
          Detection only. The platform never reproduces wash trading, spoofing or coordinated pumps.
        </p>
      </div>
      {rules.length > 0 ? (
        <div className="panel p-4">
          <div className="kicker mb-3">Rules</div>
          <div className="flex flex-wrap gap-2">
            {rules.map((r) => (
              <button
                key={r.kind}
                type="button"
                className={`font-mono text-[11px] border rounded px-2 py-1 ${
                  r.enabled ? "border-phosphor text-phosphor" : "border-line text-mute"
                }`}
                onClick={async () => {
                  await api.setAlertRule(r.kind, !r.enabled);
                  const next = await api.alertRules();
                  setRules(next.rules ?? []);
                }}
              >
                {r.kind} {r.enabled ? "ON" : "OFF"}
              </button>
            ))}
          </div>
        </div>
      ) : null}
      <form
        className="panel p-4 grid gap-3 md:grid-cols-[120px_1fr_1fr_auto]"
        onSubmit={async (e) => {
          e.preventDefault();
          await api.createWebhook({
            kind,
            url,
            chat_id: chatId || undefined,
          });
          const r = await api.webhooks();
          setHooks(r.webhooks ?? []);
          setUrl("");
          setChatId("");
        }}
      >
        <select
          className="bg-void border border-line rounded px-2 py-2 text-sm"
          value={kind}
          onChange={(e) => setKind(e.target.value)}
        >
          <option value="discord">discord</option>
          <option value="telegram">telegram</option>
        </select>
        <input
          className="bg-void border border-line rounded px-3 py-2 text-sm font-mono"
          placeholder="https://discord.com/api/webhooks/…"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
        />
        <input
          className="bg-void border border-line rounded px-3 py-2 text-sm font-mono"
          placeholder="Telegram chat_id (optional)"
          value={chatId}
          onChange={(e) => setChatId(e.target.value)}
        />
        <button className="bg-phosphor text-void text-sm rounded px-3 py-2" type="submit">
          Add webhook
        </button>
        {hooks.length > 0 ? (
          <ul className="md:col-span-3 text-xs text-mute space-y-1">
            {hooks.map((h) => (
              <li key={h.id} className="flex justify-between gap-2">
                <span className="font-mono truncate">
                  {h.kind} · {h.url}
                </span>
                <button
                  type="button"
                  className="text-danger"
                  onClick={async () => {
                    await api.deleteWebhook(h.id);
                    setHooks((prev) => prev.filter((x) => x.id !== h.id));
                  }}
                >
                  delete
                </button>
              </li>
            ))}
          </ul>
        ) : null}
      </form>
      <div className="space-y-3">
        {alerts.length === 0 ? (
          <div className="panel p-6 text-mute text-sm">No alerts on the current snapshot.</div>
        ) : (
          alerts.map((a) => (
            <div key={a.id} className="panel p-4">
              <div className="flex justify-between gap-4">
                <div>
                  <div className="font-mono text-[11px] text-mute uppercase">
                    {a.severity} · {a.kind}
                  </div>
                  <div className="text-lg mt-1">{a.title}</div>
                  <p className="text-sm text-mute mt-1">{a.body}</p>
                </div>
                <div className="flex flex-col items-end gap-2">
                  <Link href={`/tokens/${a.token_id}`} className="text-signal text-sm">
                    Open
                  </Link>
                  <button
                    type="button"
                    className="text-xs text-mute"
                    onClick={async () => {
                      await api.ackAlert(a.id);
                      setAlerts((prev) => (prev ?? []).filter((x) => x.id !== a.id));
                    }}
                  >
                    ack
                  </button>
                </div>
              </div>
              <ul className="mt-3 text-xs text-mute list-disc pl-4">
                {a.evidence.map((e) => (
                  <li key={e}>{e}</li>
                ))}
              </ul>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
