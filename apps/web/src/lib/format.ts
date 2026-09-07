export function compactUsd(n: number): string {
  const abs = Math.abs(n);
  const sign = n < 0 ? "-" : "";
  if (abs >= 1e9) return `${sign}$${(abs / 1e9).toFixed(2)}B`;
  if (abs >= 1e6) return `${sign}$${(abs / 1e6).toFixed(2)}M`;
  if (abs >= 1e3) return `${sign}$${(abs / 1e3).toFixed(1)}K`;
  if (abs >= 1) return `${sign}$${abs.toFixed(2)}`;
  if (abs === 0) return "$0";
  return `${sign}$${abs.toExponential(2)}`;
}

/** Spot price for memecoins — keeps leading zeros instead of $3.71e-6. */
export function formatPrice(n: number | null | undefined, state?: string | null): string {
  const s = (state ?? "").toLowerCase();
  if (s && s !== "live" && s !== "recent" && s !== "stale" && s !== "conflict" && s !== "simulated") {
    return "—";
  }
  if (n == null || !Number.isFinite(n) || n <= 0) return "—";
  const abs = Math.abs(n);
  const sign = n < 0 ? "-" : "";
  if (abs >= 1) return `${sign}$${abs.toFixed(2)}`;
  if (abs >= 0.01) return `${sign}$${abs.toFixed(4)}`;
  const raw = abs.toFixed(12).replace(/0+$/, "");
  return `${sign}$${raw}`;
}

export function compactInt(n: number): string {
  if (n >= 1e6) return `${(n / 1e6).toFixed(2)}M`;
  if (n >= 1e3) return `${(n / 1e3).toFixed(1)}K`;
  return n.toLocaleString();
}

export function pct(n: number, digits = 1): string {
  const sign = n > 0 ? "+" : "";
  return `${sign}${n.toFixed(digits)}%`;
}

export function scoreTone(n: number): string {
  if (n >= 70) return "text-phosphor";
  if (n >= 45) return "text-warn";
  return "text-danger";
}

export function riskTone(level: string): string {
  const l = level.toLowerCase();
  if (l.includes("low")) return "text-phosphor";
  if (l.includes("moderate")) return "text-warn";
  if (l.includes("unknown")) return "text-mute";
  return "text-danger";
}

export function labelize(value: string): string {
  return value.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function presentState(state?: string | null): boolean {
  const s = (state ?? "").toLowerCase();
  return s === "live" || s === "recent" || s === "stale" || s === "conflict" || s === "simulated";
}

/** Live number or an honest blank — never a fake zero. */
export function connectedValue(state: string | null | undefined, value: string): string {
  return presentState(state) ? value : "—";
}
