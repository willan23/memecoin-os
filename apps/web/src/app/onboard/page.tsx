"use client";

import { api } from "@/lib/api";
import { useState } from "react";
import { useRouter } from "next/navigation";

const STEPS = [
  "Identity",
  "Contract",
  "Sources",
  "Verify",
  "Index",
  "Score",
  "Agent",
];

export default function OnboardPage() {
  const router = useRouter();
  const [step, setStep] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [form, setForm] = useState({
    name: "",
    symbol: "",
    chain: "ethereum",
    address: "",
    website: "",
    decimals: 18,
  });

  async function publish() {
    setError(null);
    try {
      const snap = await api.onboard({
        ...form,
        website: form.website || undefined,
      });
      router.push(`/tokens/${snap.token.id}`);
    } catch (e) {
      setError(String((e as Error).message));
    }
  }

  return (
    <div className="max-w-2xl space-y-6">
      <div>
        <div className="kicker">Token onboarding engine</div>
        <h1 className="text-3xl mt-1">Add a memecoin</h1>
        <p className="text-sm text-mute mt-2">
          New tokens start as UNVERIFIED. Verified is not the same as safe. Production writes need the operator key in Settings.
        </p>
      </div>
      <div className="flex gap-2 text-[11px] font-mono uppercase tracking-wide">
        {STEPS.map((s, i) => (
          <div key={s} className={i <= step ? "text-phosphor" : "text-mute"}>
            {i + 1}. {s}
          </div>
        ))}
      </div>
      <div className="panel p-5 space-y-3">
        {step === 0 && (
          <>
            <Field label="Name" value={form.name} onChange={(v) => setForm({ ...form, name: v })} />
            <Field label="Symbol" value={form.symbol} onChange={(v) => setForm({ ...form, symbol: v })} />
          </>
        )}
        {step === 1 && (
          <>
            <label className="text-xs text-mute">Chain</label>
            <select
              className="w-full bg-void border border-line rounded-lg px-3 py-2 text-sm"
              value={form.chain}
              onChange={(e) => setForm({ ...form, chain: e.target.value })}
            >
              {["ethereum", "arbitrum", "bsc", "base", "polygon", "optimism", "avalanche", "solana"].map((c) => (
                <option key={c}>{c}</option>
              ))}
            </select>
            <Field label="Contract" value={form.address} onChange={(v) => setForm({ ...form, address: v })} />
            <Field
              label="Decimals"
              value={String(form.decimals)}
              onChange={(v) => setForm({ ...form, decimals: Number(v) || 18 })}
            />
          </>
        )}
        {step === 2 && (
          <Field label="Website" value={form.website} onChange={(v) => setForm({ ...form, website: v })} />
        )}
        {step >= 3 && (
          <div className="text-sm text-mute space-y-2">
            <p>Verify contract → discover pools/exchanges/social → index → score → create agent → publish.</p>
            <p>
              MVP indexes from the definition immediately. Live DEX discovery is adapter-ready and not auto-promoting.
            </p>
            <pre className="text-xs bg-void p-3 rounded border border-line overflow-auto">
{JSON.stringify(form, null, 2)}
            </pre>
          </div>
        )}
        {error ? <div className="text-danger text-sm">{error}</div> : null}
        <div className="flex gap-2 pt-2">
          {step > 0 ? (
            <button className="border border-line rounded-lg px-3 py-2 text-sm" onClick={() => setStep(step - 1)}>
              Back
            </button>
          ) : null}
          {step < STEPS.length - 1 ? (
            <button
              className="bg-phosphor text-void rounded-lg px-3 py-2 text-sm font-medium"
              onClick={() => setStep(step + 1)}
            >
              Continue
            </button>
          ) : (
            <button className="bg-phosphor text-void rounded-lg px-3 py-2 text-sm font-medium" onClick={publish}>
              Index & publish
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function Field({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <label className="block">
      <div className="text-xs text-mute mb-1">{label}</div>
      <input
        className="w-full bg-void border border-line rounded-lg px-3 py-2 text-sm"
        value={value}
        onChange={(e) => onChange(e.target.value)}
      />
    </label>
  );
}
