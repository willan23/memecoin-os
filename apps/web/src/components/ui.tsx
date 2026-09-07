import { scoreTone } from "@/lib/format";

export function Metric({
  label,
  value,
  hint,
  tone,
}: {
  label: string;
  value: string;
  hint?: string;
  tone?: string;
}) {
  return (
    <div className="panel p-4">
      <div className="kicker">{label}</div>
      <div className={`mt-2 font-mono text-2xl ${tone ?? "text-ink"}`}>{value}</div>
      {hint ? <div className="mt-2 text-xs text-mute">{hint}</div> : null}
    </div>
  );
}

export function Bar({ value, label }: { value: number; label: string }) {
  return (
    <div>
      <div className="flex justify-between text-xs mb-1">
        <span className="text-mute">{label}</span>
        <span className={`font-mono ${scoreTone(value)}`}>{value.toFixed(0)}</span>
      </div>
      <div className="h-1.5 rounded-full bg-white/5 overflow-hidden">
        <div
          className="h-full bg-phosphor/80"
          style={{ width: `${Math.max(2, Math.min(100, value))}%` }}
        />
      </div>
    </div>
  );
}

export function ErrorState({ message }: { message: string }) {
  return (
    <div className="panel p-8 text-sm text-mute">
      <div className="text-danger font-medium mb-2">API unavailable</div>
      {message}. Start the Rust API from the repo root with{" "}
      <code className="text-phosphor">cargo run -p memecoin-os-api</code>.
    </div>
  );
}

export function DataStateBadge({ state }: { state?: string | null }) {
  const s = (state ?? "missing").toLowerCase();
  const label = s === "missing" ? "not connected" : s;
  const tone =
    s === "live" || s === "recent"
      ? "text-phosphor border-phosphor/40"
      : s === "conflict"
        ? "text-danger border-danger/40"
        : s === "simulated"
          ? "text-amber-300 border-amber-300/40"
          : "text-mute border-line";
  return (
    <span className={`font-mono text-[10px] uppercase tracking-wide border rounded px-1.5 py-0.5 ${tone}`}>
      {label}
    </span>
  );
}

export function MissingHint({ label }: { label: string }) {
  return (
    <div className="panel p-4 text-sm text-mute">
      <span className="font-mono text-[10px] uppercase text-mute mr-2">not connected</span>
      {label}
    </div>
  );
}
