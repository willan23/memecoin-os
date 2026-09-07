"use client";

import { api } from "@/lib/api";
import {
  Activity,
  Bell,
  Bookmark,
  Briefcase,
  FileText,
  GitCompare,
  LayoutDashboard,
  Network,
  Plus,
  Puzzle,
  Radar,
  Settings,
  ScanSearch,
  SlidersHorizontal,
  Sparkles,
  Trophy,
  Users,
  Waves,
  LogIn,
} from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState, type CSSProperties } from "react";

const NAV = [
  { href: "/", label: "Overview", icon: LayoutDashboard },
  { href: "/watchlist", label: "Watchlist", icon: Bookmark },
  { href: "/twin", label: "Digital Twin", icon: Network },
  { href: "/briefing", label: "Briefing", icon: FileText },
  { href: "/tokens", label: "Ecosystems", icon: Activity },
  { href: "/discovery", label: "Discovery", icon: ScanSearch },
  { href: "/narratives", label: "Narratives", icon: Waves },
  { href: "/simulations", label: "Simulations", icon: SlidersHorizontal },
  { href: "/compare", label: "Compare", icon: GitCompare },
  { href: "/rankings", label: "MemeCoin 100", icon: Trophy },
  { href: "/community", label: "Community", icon: Users },
  { href: "/alerts", label: "Alerts", icon: Bell },
  { href: "/intelligence", label: "AI Agent", icon: Sparkles },
  { href: "/projects", label: "Projects", icon: Briefcase },
  { href: "/exchange", label: "Exchange API", icon: Puzzle },
  { href: "/onboard", label: "Onboard", icon: Plus },
  { href: "/status", label: "Status", icon: Radar },
  { href: "/login", label: "Sign in", icon: LogIn },
  { href: "/settings", label: "Settings", icon: Settings },
];

export function Shell({ children }: { children: React.ReactNode }) {
  const path = usePathname();
  if (path.startsWith("/embed")) {
    return <>{children}</>;
  }
  const brief = path === "/" || path.startsWith("/tokens/");
  const [brand, setBrand] = useState<{
    display_name?: string | null;
    logo_url?: string | null;
    accent?: string | null;
  }>({});
  useEffect(() => {
    api
      .branding()
      .then((d) => setBrand(d.branding ?? {}))
      .catch(() => setBrand({}));
  }, []);
  const title = brand.display_name || "MEMECOIN OS";
  const accent = brand.accent || undefined;
  return (
    <div
      className="min-h-screen md:grid md:grid-cols-[240px_1fr] pb-16 md:pb-0"
      style={accent ? ({ ["--phosphor" as string]: accent } as CSSProperties) : undefined}
    >
      <aside className="hidden md:flex border-r border-line bg-[#070910]/90 px-4 py-6 flex-col">
        <Link href="/" className="px-2 mb-8">
          {brand.logo_url ? (
            // eslint-disable-next-line @next/next/no-img-element
            <img src={brand.logo_url} alt="" className="h-8 mb-2 object-contain" />
          ) : null}
          <div className="font-mono text-[11px] tracking-[0.28em] text-phosphor">
            {title}
          </div>
          <div className="text-sm text-mute mt-1">Intelligence infrastructure</div>
        </Link>
        <nav className="space-y-1 flex-1">
          {NAV.map((item) => {
            const active =
              item.href === "/" ? path === "/" : path.startsWith(item.href);
            const Icon = item.icon;
            return (
              <Link
                key={item.href}
                href={item.href}
                className={`flex items-center gap-2 rounded-lg px-3 py-2 text-sm ${
                  active
                    ? "bg-phosphor/10 text-phosphor"
                    : "text-mute hover:text-ink hover:bg-white/5"
                }`}
              >
                <Icon size={16} />
                {item.label}
              </Link>
            );
          })}
        </nav>
        <div className="mt-6 rounded-lg border border-line p-3 text-[11px] text-mute leading-relaxed">
          Scores describe observable quality. They are not price forecasts or
          financial advice. EXECUTE is disabled.
        </div>
      </aside>
      <main className="min-w-0">
        <header className="h-14 border-b border-line flex items-center justify-between px-4 md:px-6">
          <div className="flex items-center gap-2 text-mute text-xs font-mono">
            <Radar size={14} className="text-phosphor" />
            <span className="md:hidden">{title}</span>
            <span className="hidden md:inline">LIVE TERMINAL</span>
            {brief ? (
              <span className="md:hidden text-phosphor/80">· brief</span>
            ) : null}
          </div>
          <HeaderTokens />
        </header>
        <div className="p-4 md:p-6">{children}</div>
      </main>
      <nav className="md:hidden fixed bottom-0 inset-x-0 border-t border-line bg-[#070910]/95 grid grid-cols-4 text-[10px]">
        {NAV.filter((n) =>
          ["/", "/twin", "/alerts", "/status"].includes(n.href)
        ).map((item) => {
          const active =
            item.href === "/" ? path === "/" : path.startsWith(item.href);
          const Icon = item.icon;
          return (
            <Link
              key={item.href}
              href={item.href}
              className={`flex flex-col items-center py-2 ${
                active ? "text-phosphor" : "text-mute"
              }`}
            >
              <Icon size={16} />
              {item.label}
            </Link>
          );
        })}
      </nav>
    </div>
  );
}

function HeaderTokens() {
  const [symbols, setSymbols] = useState<string[]>([]);
  useEffect(() => {
    api
      .overview()
      .then((o) => setSymbols(o.tokens.map((t) => t.token.symbol)))
      .catch(() => setSymbols([]));
  }, []);
  if (!symbols.length) return null;
  return (
    <div className="text-xs text-mute truncate hidden sm:block">{symbols.join(" · ")}</div>
  );
}
