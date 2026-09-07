"use client";

import { api } from "@/lib/api";
import { operatorKey, setOperatorKey } from "@/lib/operator";
import { ErrorState } from "@/components/ui";
import { useEffect, useState } from "react";

type Flag = { key: string; enabled: boolean; reason?: string | null };
type KeyRow = { id: string; name: string; prefix: string; role: string; revoked: boolean };
type Billing = Awaited<ReturnType<typeof api.billing>>;
type Queue = Awaited<ReturnType<typeof api.queue>>;
type Sso = Awaited<ReturnType<typeof api.sso>>;
type TenantRow = { id: string; slug: string; name: string; plan: string; status: string };

export default function SettingsPage() {
  const [flags, setFlags] = useState<Flag[]>([]);
  const [keys, setKeys] = useState<KeyRow[]>([]);
  const [authRequired, setAuthRequired] = useState(false);
  const [secret, setSecret] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [keyName, setKeyName] = useState("sdk");
  const [billing, setBilling] = useState<Billing | null>(null);
  const [queue, setQueue] = useState<Queue | null>(null);
  const [sso, setSso] = useState<Sso | null>(null);
  const [tenants, setTenants] = useState<TenantRow[]>([]);
  const [watch, setWatch] = useState<string[]>([]);
  const [ecos, setEcos] = useState<{ id: string; symbol: string; name: string }[]>([]);
  const [watchSaved, setWatchSaved] = useState(false);
  const [watchErr, setWatchErr] = useState<string | null>(null);
  const [brandName, setBrandName] = useState("");
  const [brandLogo, setBrandLogo] = useState("");
  const [brandAccent, setBrandAccent] = useState("");
  const [brandDomain, setBrandDomain] = useState("");
  const [opKey, setOpKey] = useState("");
  const [opSaved, setOpSaved] = useState(false);

  async function reload() {
    const [f, k, b, q, s, t, w, ecos, br] = await Promise.all([
      api.flags(),
      api.keys(),
      api.billing(),
      api.queue(),
      api.sso(),
      api.tenants(),
      api.watchlist().catch(() => ({ token_ids: [] as string[] })),
      api.overview().catch(() => ({ tokens: [] as { token: { id: string; symbol: string; name: string } }[] })),
      api.branding().catch(() => ({
        branding: { display_name: null, logo_url: null, accent: null, custom_domain: null },
      })),
    ]);
    const list = Array.isArray(f) ? f : (f as { flags?: Flag[] }).flags ?? [];
    setFlags(list);
    setKeys(k.keys ?? []);
    setAuthRequired(Boolean(k.auth_required));
    setBilling(b);
    setQueue(q);
    setSso(s);
    setTenants(t.tenants ?? []);
    setWatch(w.token_ids ?? []);
    setEcos(
      (ecos.tokens ?? []).map((t) => ({
        id: t.token.id,
        symbol: t.token.symbol,
        name: t.token.name,
      }))
    );
    setWatchSaved(false);
    setWatchErr(null);
    setBrandName(br.branding?.display_name ?? "");
    setBrandLogo(br.branding?.logo_url ?? "");
    setBrandAccent(br.branding?.accent ?? "");
    setBrandDomain(br.branding?.custom_domain ?? "");
  }

  useEffect(() => {
    setOpKey(operatorKey());
    setOpSaved(Boolean(operatorKey()));
    reload().catch((e) => setError(String(e.message ?? e)));
  }, []);

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-8 max-w-3xl">
      <div>
        <div className="kicker">Operations</div>
        <h1 className="text-3xl mt-1">Settings</h1>
        <p className="text-sm text-mute mt-2">
          Feature flags, API keys, plan usage and SSO status. EXECUTE stays off. No fake charges.
        </p>
      </div>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Operator write key</div>
        <p className="text-xs text-mute">
          Production Track / onboard / webhook create / project claim and official links require
          this key. Discord/Telegram click-to-connect uses DISCORD_* / TELEGRAM_* on the API, not
          this browser key. It stays in this browser session only — not in the static site, not in
          NEXT_PUBLIC.
        </p>
        <div className="flex gap-2">
          <input
            type="password"
            autoComplete="off"
            className="flex-1 bg-void border border-line rounded px-3 py-2 text-sm font-mono"
            value={opKey}
            onChange={(e) => {
              setOpKey(e.target.value);
              setOpSaved(false);
            }}
            placeholder="OPERATOR_API_KEY from local .env"
          />
          <button
            className="bg-phosphor text-void text-sm rounded px-3 py-2"
            onClick={() => {
              setOperatorKey(opKey);
              setOpSaved(Boolean(opKey.trim()));
            }}
          >
            Save session
          </button>
        </div>
        <div className="text-xs text-mute">{opSaved ? "Saved in this session." : "Not saved in this browser."}</div>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Plan & usage</div>
        {billing?.degraded ? (
          <div className="text-sm text-mute">Postgres down — metering unavailable.</div>
        ) : (
          <dl className="grid grid-cols-2 gap-2 text-sm">
            <dt className="text-mute">Plan</dt>
            <dd className="font-mono">{billing?.plan ?? "—"}</dd>
            <dt className="text-mute">Enforce quotas</dt>
            <dd className="font-mono">{billing?.metering_enforce ? "on" : "off"}</dd>
            <dt className="text-mute">API today</dt>
            <dd className="font-mono">
              {billing?.usage_today?.api_requests ?? 0} / {fmtLimit(billing?.limits?.api_requests_day)}
            </dd>
            <dt className="text-mute">Research today</dt>
            <dd className="font-mono">
              {billing?.usage_today?.research_reports ?? 0} / {fmtLimit(billing?.limits?.research_day)}
            </dd>
            <dt className="text-mute">AI today</dt>
            <dd className="font-mono">
              {billing?.usage_today?.ai_requests ?? 0} / {fmtLimit(billing?.limits?.ai_requests_day)}
            </dd>
          </dl>
        )}
        <p className="text-xs text-mute">{billing?.note}</p>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Gerenciamento de pagamento</div>
        <p className="text-sm text-mute">
          Planos, quotas e o estado do processador. Sem atalho para a conta do operador. Pagar nunca
          muda um score.
        </p>
        <dl className="grid grid-cols-2 gap-2 text-sm">
          <dt className="text-mute">Processador</dt>
          <dd className="font-mono">Stripe</dd>
          <dt className="text-mute">Estado</dt>
          <dd className="font-mono">
            {billing?.stripe?.configured ? (
              <span className="text-phosphor">ligado</span>
            ) : (
              <span className="text-mute">meters only — keys no .env</span>
            )}
          </dd>
          <dt className="text-mute">Checkout</dt>
          <dd className="font-mono">{billing?.stripe?.checkout ?? "meters_only"}</dd>
          <dt className="text-mute">Planos mapeados</dt>
          <dd className="font-mono text-xs">
            {(["pro", "research", "growth", "enterprise", "api"] as const).map((k) => (
              <span key={k} className="mr-2">
                {k}: {billing?.stripe?.prices_mapped?.[k] ? "ok" : "—"}
              </span>
            ))}
          </dd>
        </dl>
        <p className="text-xs text-mute">
          {billing?.stripe?.configured
            ? "Webhooks assinados atualizam o plano. Sem cobrança inventada."
            : "STRIPE_SECRET_KEY e STRIPE_WEBHOOK_SECRET ficam só no .env local / Cloud Run. Nunca no chat."}
        </p>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">White-label</div>
        <p className="text-xs text-mute">Chrome only. HTTPS logo. Twin facts stay shared.</p>
        <div className="grid gap-2 text-sm">
          <input className="bg-void border border-line rounded px-2 py-1" placeholder="Display name" value={brandName} onChange={(e) => setBrandName(e.target.value)} />
          <input className="bg-void border border-line rounded px-2 py-1" placeholder="https://… logo" value={brandLogo} onChange={(e) => setBrandLogo(e.target.value)} />
          <input className="bg-void border border-line rounded px-2 py-1" placeholder="#3EE58A" value={brandAccent} onChange={(e) => setBrandAccent(e.target.value)} />
          <input className="bg-void border border-line rounded px-2 py-1" placeholder="twin.example.com" value={brandDomain} onChange={(e) => setBrandDomain(e.target.value)} />
        </div>
        <button
          className="bg-phosphor text-void text-sm rounded px-3 py-1"
          onClick={async () => {
            const id = billing?.tenant_id ?? tenants[0]?.id;
            if (!id) {
              setError("No tenant (Postgres).");
              return;
            }
            try {
              await api.saveBranding(id, {
                display_name: brandName,
                logo_url: brandLogo,
                accent: brandAccent,
                custom_domain: brandDomain,
              });
              await reload();
            } catch (e) {
              setError(String((e as Error).message));
            }
          }}
        >
          Save branding
        </button>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Watchlist</div>
        <p className="text-xs text-mute">
          Personal filter for Overview, Ecosystems and{" "}
          <a href="/watchlist" className="text-phosphor underline">
            /watchlist
          </a>
          . Server-side isolation only when AUTH_REQUIRED. Snapshots stay shared. Not a forked Twin.
        </p>
        {ecos.length === 0 ? (
          <div className="text-sm text-mute">No catalog yet — Postgres or refresh first.</div>
        ) : (
          <ul className="space-y-1 text-sm">
            {ecos.map((e) => (
              <li key={e.id}>
                <label className="flex gap-2 items-center">
                  <input
                    type="checkbox"
                    checked={watch.includes(e.id)}
                    onChange={() => {
                      setWatchSaved(false);
                      setWatch((cur) => (cur.includes(e.id) ? cur.filter((x) => x !== e.id) : [...cur, e.id]));
                    }}
                  />
                  <span className="font-mono text-phosphor text-xs">{e.symbol}</span>
                  <span className="text-sm">{e.name}</span>
                  <span className="font-mono text-[11px] text-mute">{e.id}</span>
                </label>
              </li>
            ))}
          </ul>
        )}
        <div className="flex items-center gap-3">
          <button
            className="border border-line text-sm rounded px-3 py-1"
            onClick={async () => {
              try {
                setWatchErr(null);
                await api.saveWatchlist(watch);
                setWatchSaved(true);
              } catch (e) {
                setWatchErr(String((e as Error).message));
              }
            }}
          >
            Save watchlist
          </button>
          <span className="text-xs text-mute">{watch.length} selected</span>
        </div>
        {watchSaved ? <div className="text-xs text-phosphor">Saved.</div> : null}
        {watchErr ? <div className="text-xs text-danger">{watchErr}</div> : null}
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Queue</div>
        <div className="text-sm font-mono">
          pending {queue?.pending ?? "—"} · locked {queue?.locked ?? "—"}
        </div>
        <p className="text-xs text-mute">{queue?.note}</p>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">SSO</div>
        <div className="text-sm">
          {sso?.configured ? (
            <span>
              Configured · issuer <span className="font-mono text-xs">{sso.issuer}</span>
            </span>
          ) : (
            <span className="text-mute">Not configured — no local fake users.</span>
          )}
        </div>
        {sso?.configured ? (
          <a className="text-sm text-phosphor underline" href="/v1/auth/oidc/login">
            Sign in with IdP
          </a>
        ) : null}
        <p className="text-xs text-mute">{sso?.note}</p>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Tenants</div>
        <p className="text-xs text-mute">
          Token snapshots stay shared public data. Isolation is keys, usage, queue and members.
        </p>
        <ul className="space-y-1 text-xs font-mono">
          {tenants.map((t) => (
            <li key={t.id}>
              {t.slug} · {t.plan} · {t.status}
            </li>
          ))}
        </ul>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Jobs</div>
        <button
          disabled={busy}
          className="bg-phosphor text-void text-sm rounded px-3 py-2 disabled:opacity-50"
          onClick={async () => {
            setBusy(true);
            try {
              await api.refreshAll();
            } catch (e) {
              setError(String((e as Error).message));
            } finally {
              setBusy(false);
            }
          }}
        >
          {busy ? "Refreshing…" : "Refresh all snapshots"}
        </button>
        <p className="text-xs text-mute">
          Respects per-token hourly quota. Holders use Ethplorer on Ethereum; other chains need
          ETHERSCAN_API_KEY. Mention firehose stays not connected without a licence. Chat is already live.
        </p>
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">Feature flags</div>
        {flags.length === 0 ? (
          <div className="text-sm text-mute">No flags (Postgres down — env fallback only).</div>
        ) : (
          <ul className="space-y-2">
            {flags.map((f) => (
              <li key={f.key} className="flex items-center justify-between gap-3 text-sm">
                <div>
                  <div className="font-mono text-xs">{f.key}</div>
                  {f.reason ? <div className="text-xs text-mute">{f.reason}</div> : null}
                </div>
                <button
                  className={`font-mono text-xs border rounded px-2 py-1 ${
                    f.enabled ? "border-phosphor text-phosphor" : "border-line text-mute"
                  }`}
                  onClick={async () => {
                    await api.setFlag(f.key, !f.enabled);
                    await reload();
                  }}
                >
                  {f.enabled ? "ON" : "OFF"}
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className="panel p-5 space-y-3">
        <div className="kicker">API keys</div>
        <p className="text-xs text-mute">
          Optional. AUTH_REQUIRED is {authRequired ? "on" : "off"}. Secrets are shown once.
        </p>
        <form
          className="flex gap-2"
          onSubmit={async (e) => {
            e.preventDefault();
            const created = await api.createKey(keyName);
            setSecret(created.secret);
            await reload();
          }}
        >
          <input
            className="flex-1 bg-void border border-line rounded px-3 py-2 text-sm"
            value={keyName}
            onChange={(e) => setKeyName(e.target.value)}
            placeholder="key name"
          />
          <button className="bg-phosphor text-void text-sm rounded px-3 py-2" type="submit">
            Create
          </button>
        </form>
        {secret ? (
          <pre className="text-xs bg-void border border-line rounded p-3 overflow-auto">{secret}</pre>
        ) : null}
        <ul className="space-y-2 text-xs font-mono">
          {keys.map((k) => (
            <li key={k.id} className="flex justify-between gap-2">
              <span className={k.revoked ? "text-mute line-through" : ""}>
                {k.name} · {k.prefix}… · {k.role}
              </span>
              {!k.revoked ? (
                <button
                  className="text-danger"
                  onClick={async () => {
                    await api.revokeKey(k.id);
                    await reload();
                  }}
                >
                  revoke
                </button>
              ) : (
                <span className="text-mute">revoked</span>
              )}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

function fmtLimit(n?: number) {
  if (n === undefined) return "—";
  if (n < 0) return "∞";
  return String(n);
}
