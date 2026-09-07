"use client";

import { api } from "@/lib/api";
import { compactInt, compactUsd, connectedValue, formatPrice, labelize, pct, presentState, riskTone, scoreTone } from "@/lib/format";
import type { TokenSnapshot } from "@/lib/types";
import { Bar, DataStateBadge, ErrorState, Metric, MissingHint } from "@/components/ui";
import Link from "next/link";
import { useEntityId } from "@/lib/route-id";
import { useEffect, useState } from "react";
import {
  Area,
  AreaChart,
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
} from "recharts";

const TABS = [
  "Overview",
  "Markets",
  "On-chain",
  "Social",
  "Development",
  "Risk",
  "Growth",
  "Genome",
  "Timeline",
  "Baselines",
  "Claims",
  "AI",
  "History",
] as const;

export default function TokenPage() {
  const id = useEntityId();
  const [tab, setTab] = useState<(typeof TABS)[number]>("Overview");
  const [data, setData] = useState<TokenSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [question, setQuestion] = useState("Analyze our ecosystem. What should the community investigate?");
  const [answer, setAnswer] = useState<string | null>(null);
  const [refreshing, setRefreshing] = useState(false);
  const [computedVerification, setComputedVerification] = useState<string | null>(null);
  const [official, setOfficial] = useState<{
    website: string;
    twitter: string;
    telegram: string;
    discord: string;
    github: string;
  } | null>(null);

  useEffect(() => {
    if (!id) return;
    setData(null);
    setError(null);
    api.token(id).then(setData).catch((e) => setError(String(e.message ?? e)));
    api.verification(id).then((v) => setComputedVerification(v.level)).catch(() => setComputedVerification(null));
    api.project(id).then((p) => setOfficial(p.official)).catch(() => setOfficial(null));
  }, [id]);

  if (error) return <ErrorState message={error} />;
  if (!data) return <div className="text-mute text-sm">Building snapshot…</div>;

  return (
    <div className="space-y-6">
      <div className="flex items-start justify-between gap-6">
        <div>
          <div className="kicker">
            {data.token.primary_chain} · declared {labelize(data.token.verification)}
            {computedVerification ? ` · computed ${labelize(computedVerification)}` : ""}
            {" · VERIFIED ≠ SAFE"}
          </div>
          <h1 className="text-3xl mt-1">
            {data.token.name} <span className="text-mute font-mono text-xl">{data.token.symbol}</span>
          </h1>
          <div className="mt-2 flex gap-2 items-center">
            <DataStateBadge state={data.data_state} />
            <DataStateBadge state={data.market.data_state} />
            <a href={`/v1/tokens/${data.token.id}/briefing.md`} className="text-xs text-signal font-mono">
              briefing.md
            </a>
            <button
              disabled={refreshing}
              className="text-xs font-mono text-signal disabled:opacity-50"
              onClick={async () => {
                setRefreshing(true);
                try {
                  const next = await api.refreshToken(data.token.id);
                  setData(next);
                } finally {
                  setRefreshing(false);
                }
              }}
            >
              {refreshing ? "refreshing…" : "refresh"}
            </button>
          </div>
          <p className="text-sm text-mute mt-2 max-w-2xl">{data.token.description}</p>
          <div className="mt-3 flex flex-wrap gap-2 text-xs">
            {data.token.narratives.map((n) => (
              <span key={n} className="border border-line rounded px-2 py-0.5 text-mute">{n}</span>
            ))}
            {data.token.contract ? (
              <span className="font-mono text-mute">{data.token.contract.slice(0, 10)}…{data.token.contract.slice(-4)}</span>
            ) : null}
          </div>
        </div>
        <div className="text-right">
          <div className="kicker">Ecosystem health</div>
          <div className={`font-mono text-5xl ${scoreTone(data.scores.value)}`}>{data.scores.value.toFixed(0)}</div>
          <div className="text-xs text-mute mt-1">
            conf {(data.scores.confidence * 100).toFixed(0)}% · {data.scores.algorithm} {data.scores.algorithm_version}
          </div>
        </div>
      </div>

      <div className="flex gap-1 border-b border-line">
        {TABS.map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`px-3 py-2 text-sm ${
              tab === t ? "text-phosphor border-b border-phosphor" : "text-mute"
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      {tab === "Overview" && (
        <div className="space-y-4">
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <Metric label="Price" value={formatPrice(data.market.price_usd, data.market.data_state)} hint={data.market.provenance.provider} />
            <Metric label="Market cap" value={compactUsd(data.market.market_cap_usd)} />
            <Metric label="24h volume" value={compactUsd(data.market.volume_24h_usd)} tone={data.market.change_24h_pct >= 0 ? "text-phosphor" : "text-danger"} hint={pct(data.market.change_24h_pct)} />
            <Metric label="Liquidity" value={compactUsd(data.liquidity.liquidity_usd)} hint={`${data.liquidity.spread_bps.toFixed(0)} bps spread`} />
            <Metric label="Risk" value={`${data.risk.score.toFixed(0)}`} tone={riskTone(data.risk.level)} hint={data.risk.level} />
            <Metric
              label="Holders"
              value={connectedValue(data.onchain.data_state, compactInt(data.onchain.holders))}
              hint={presentState(data.onchain.data_state) ? data.onchain.provenance.provider : "waiting on public holder source"}
            />
            <Metric label="Community chat" value="LIVE" hint="First-party room" />
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="panel p-4">
              <div className="kicker mb-3">Price series (persisted snapshots, simulated excluded)</div>
              <div className="h-40">
                <ResponsiveContainer>
                  <AreaChart data={data.price_series}>
                    <XAxis dataKey="t" hide />
                    <Tooltip contentStyle={{ background: "#0d1018", border: "1px solid #1c2333" }} />
                    <Area type="monotone" dataKey="v" stroke="#4da6ff" fill="rgba(77,166,255,0.15)" />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>
            <div className="panel p-4">
              <div className="kicker mb-3">Genome</div>
              <div className="h-40">
                <ResponsiveContainer>
                  <RadarChart data={data.genome.dimensions.map((d) => ({ k: d.label, v: d.value }))}>
                    <PolarGrid stroke="#1c2333" />
                    <PolarAngleAxis dataKey="k" tick={{ fill: "#8b95a8", fontSize: 9 }} />
                    <Radar dataKey="v" stroke="#3ee58a" fill="rgba(62,229,138,0.25)" />
                  </RadarChart>
                </ResponsiveContainer>
              </div>
            </div>
          </div>
          <div className="grid grid-cols-2 gap-3">
            <div className="panel p-4 space-y-3">
              <div className="kicker">Why this score</div>
              <p className="text-sm text-mute">{data.scores.why}</p>
              {data.scores.components.map((c) => (
                <Bar key={c.id} label={`${c.label} · w${(c.weight * 100).toFixed(0)}%`} value={c.value} />
              ))}
            </div>
            <div className="panel p-4">
              <div className="kicker mb-3">Timeline</div>
              <ol className="space-y-3">
                {data.timeline.map((e) => (
                  <li key={e.at + e.title} className="text-sm border-l border-line pl-3">
                    <div className="font-mono text-[11px] text-mute">
                      {new Date(e.at).toLocaleString()} · {e.kind}
                    </div>
                    <div>{e.title}</div>
                  </li>
                ))}
              </ol>
            </div>
          </div>
        </div>
      )}

      {tab === "Markets" && (
        <div className="grid grid-cols-3 gap-3">
          <div className="col-span-3 flex gap-2">
            <DataStateBadge state={data.market.data_state} />
            <DataStateBadge state={data.liquidity.data_state} />
          </div>
          <Metric label="FDV" value={compactUsd(data.market.fdv_usd)} />
          <Metric label="ATH distance" value={data.market.ath_distance_pct != null ? pct(data.market.ath_distance_pct) : "—"} />
          <Metric label="LP 7d" value={pct(data.liquidity.lp_change_7d_pct)} />
          <Metric label="Depth +2%" value={compactUsd(data.liquidity.depth_plus_2pct_usd)} />
          <Metric label="Depth -2%" value={compactUsd(data.liquidity.depth_minus_2pct_usd)} />
          <Metric label="Pools" value={String(data.liquidity.pool_count)} />
          <div className="panel p-4 col-span-3 text-xs text-mute">
            Market health is liquidity, volume, volatility, depth, spread and stability — not a price target.
            Source {data.market.provenance.provider}, confidence {(data.market.provenance.confidence * 100).toFixed(0)}%, status {data.market.provenance.validation_status}.
          </div>
        </div>
      )}

      {tab === "On-chain" && (
        data.onchain.data_state && data.onchain.data_state !== "live" && data.onchain.data_state !== "recent" && data.onchain.data_state !== "simulated" ? (
          <div className="space-y-3">
            <MissingHint label="Holders — Ethplorer on Ethereum, Etherscan key on other EVM. Solana stays blank until an indexer RPC is set. Not a zero." />
            <TokenOnchainIndex id={data.token.id} />
          </div>
        ) : (
        <div className="grid grid-cols-4 gap-3">
          <Metric label="Holders" value={compactInt(data.onchain.holders)} hint={data.onchain.provenance.provider} />
          <Metric
            label="Top 10"
            value={data.onchain.top10_concentration_pct > 0 ? `${data.onchain.top10_concentration_pct.toFixed(1)}%` : "—"}
            hint={data.onchain.top10_concentration_pct > 0 ? undefined : "distribution not sampled"}
          />
          <Metric
            label="Whales ≥1%"
            value={data.onchain.top10_concentration_pct > 0 ? String(data.onchain.whale_holders) : "—"}
          />
          <Metric label="Top 50" value={data.onchain.top50_concentration_pct > 0 ? `${data.onchain.top50_concentration_pct.toFixed(1)}%` : "—"} />
          {data.onchain.transfers_24h === 0 ? (
            <div className="col-span-4">
              <MissingHint label="Transfers / CEX flow need an RPC indexer. Holder counts are not transfers." />
            </div>
          ) : (
            <>
              <Metric label="Transfers 24h" value={compactInt(data.onchain.transfers_24h)} />
              <Metric label="Unique senders 24h" value={compactInt(data.onchain.unique_senders_24h)} />
              <Metric label="CEX inflow" value={data.onchain.exchange_inflow_usd > 0 ? compactUsd(data.onchain.exchange_inflow_usd) : "—"} />
              <Metric label="CEX outflow" value={data.onchain.exchange_outflow_usd > 0 ? compactUsd(data.onchain.exchange_outflow_usd) : "—"} />
            </>
          )}
          <div className="col-span-4">
            <TokenOnchainIndex id={data.token.id} />
          </div>
          <div className="panel p-4 col-span-4 text-sm text-mute">
            Observed → correlated → hypothesis. A whale transfer is not a price prediction.
          </div>
        </div>
        )
      )}

      {tab === "Social" && (
        <div className="space-y-3">
          <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
            <Metric label="Platform chat" value="LIVE" hint="Write in Community — this is the live room" />
            <Metric
              label="Mention firehose"
              value={presentState(data.social.data_state) ? compactInt(data.social.mentions_24h) : "—"}
              hint={presentState(data.social.data_state) ? "licensed firehose" : "licence not connected"}
            />
            <Metric
              label="Organicness"
              value={presentState(data.social.data_state) ? data.social.organicness.toFixed(2) : "—"}
              hint="Not invented from chat or follower counts"
            />
          </div>
          <div className="panel p-4 space-y-2 text-sm">
            <div className="kicker">Official rooms</div>
            {official && [official.website, official.twitter, official.telegram, official.discord, official.github].some(Boolean) ? (
              <ul className="space-y-1">
                {official.website ? <li><a className="text-phosphor underline" href={official.website} target="_blank" rel="noreferrer">Website</a></li> : null}
                {official.twitter ? <li><a className="text-phosphor underline" href={official.twitter} target="_blank" rel="noreferrer">X / Twitter</a></li> : null}
                {official.telegram ? <li><a className="text-phosphor underline" href={official.telegram} target="_blank" rel="noreferrer">Telegram</a></li> : null}
                {official.discord ? <li><a className="text-phosphor underline" href={official.discord} target="_blank" rel="noreferrer">Discord</a></li> : null}
                {official.github ? <li><a className="text-phosphor underline" href={official.github} target="_blank" rel="noreferrer">GitHub</a></li> : null}
              </ul>
            ) : (
              <p className="text-mute">Add official links on the project desk. Empty is not a dead community.</p>
            )}
            <Link className="text-phosphor underline" href={`/community?room=${encodeURIComponent(data.token.id)}`}>
              Open live chat
            </Link>
          </div>
          {presentState(data.social.data_state) ? (
            <div className="grid grid-cols-4 gap-3">
              <Metric label="Unique accounts" value={compactInt(data.social.unique_accounts_24h)} />
              <Metric label="Bot probability" value={data.social.bot_probability.toFixed(2)} />
              <Metric label="Sentiment net" value={data.social.sentiment_net.toFixed(2)} />
              <Metric label="Narrative diversity" value={data.social.narrative_diversity.toFixed(2)} />
            </div>
          ) : (
            <MissingHint label="X/Telegram mention counts stay blank without a firehose licence. Chat above is already live." />
          )}
        </div>
      )}

      {tab === "Development" && (
        presentState(data.development.data_state) ? (
        <div className="grid grid-cols-4 gap-3">
          <div className="col-span-4">
            <DataStateBadge state={data.development.data_state} />
          </div>
          <Metric label="Commits 30d" value={String(data.development.commits_30d)} />
          <Metric label="Contributors" value={String(data.development.active_contributors_30d)} />
          <Metric label="Releases 90d" value={String(data.development.releases_90d)} />
          <Metric label="Last commit" value={data.development.last_commit_days != null ? `${data.development.last_commit_days}d` : "n/a"} />
        </div>
        ) : (
          <MissingHint label="Development — add an official GitHub URL on the project desk. Zeros here would be invented." />
        )
      )}

      {tab === "Risk" && (
        <div className="space-y-4">
          <DataStateBadge state={data.data_state} />
          <p className="text-sm text-mute">{data.risk.why}</p>
          <div className="grid grid-cols-2 gap-3">
            {data.risk.factors.map((f) => (
              <div key={f.id} className="panel p-4">
                <div className="flex justify-between">
                  <div>{f.label}</div>
                  <div className={`font-mono ${riskTone(f.level)}`}>{f.score.toFixed(0)} {f.level}</div>
                </div>
                <ul className="mt-2 text-xs text-mute space-y-1">
                  {f.evidence.map((e) => (
                    <li key={e}>{e}</li>
                  ))}
                </ul>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === "Growth" && (
        <div className="space-y-3">
          {data.growth.map((o) => (
            <div key={o.id} className="panel p-4">
              <div className="flex justify-between gap-4">
                <div>
                  <div className="font-medium">{o.title}</div>
                  <div className="text-xs text-mute mt-1">Expected metric: {o.expected_metric}</div>
                </div>
                <div className="text-right font-mono text-sm">
                  <div className="text-phosphor">impact {o.impact.toFixed(0)}</div>
                  <div className="text-mute">cost {o.cost} · risk {o.risk}</div>
                </div>
              </div>
              <ul className="mt-3 text-xs text-mute list-disc pl-4">
                {o.evidence.map((e) => (
                  <li key={e}>{e}</li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}

      {tab === "AI" && (
        <div className="grid grid-cols-2 gap-4">
          <div className="panel p-4 space-y-3">
            <div className="kicker">{data.briefing.headline}</div>
            <div>
              <div className="text-xs text-mute mb-1">Changes</div>
              <ul className="text-sm space-y-1">{data.briefing.changes.map((c) => <li key={c}>{c}</li>)}</ul>
            </div>
            <div>
              <div className="text-xs text-mute mb-1">Risks</div>
              <ul className="text-sm space-y-1">{data.briefing.risks.map((c) => <li key={c}>{c}</li>)}</ul>
            </div>
            <div>
              <div className="text-xs text-mute mb-1">Investigate</div>
              <p className="text-sm">{data.briefing.investigation}</p>
            </div>
            {data.observations.map((o) => (
              <div key={o.observation} className="border-t border-line pt-3 text-sm">
                <div>{o.observation}</div>
                <div className="text-mute text-xs mt-1">{o.interpretation} · conf {(o.confidence * 100).toFixed(0)}%</div>
              </div>
            ))}
          </div>
          <div className="panel p-4 space-y-3">
            <div className="kicker">Ecosystem agent · READ/ANALYZE/RECOMMEND</div>
            <textarea
              className="w-full h-28 bg-void border border-line rounded-lg p-3 text-sm"
              value={question}
              onChange={(e) => setQuestion(e.target.value)}
            />
            <button
              className="bg-phosphor text-void text-sm font-medium rounded-lg px-3 py-2"
              onClick={async () => {
                const r = await api.ask(question, data.token.id);
                setAnswer(r.answer);
              }}
            >
              Ask agent
            </button>
            {answer ? <pre className="whitespace-pre-wrap text-sm text-mute">{answer}</pre> : null}
            <a href={`/v1/tokens/${data.token.id}/research.md`} className="text-xs font-mono text-signal">research.md</a>
            <Link href="/intelligence" className="text-xs text-signal">Open research agent →</Link>
          </div>
        </div>
      )}

      {tab === "Genome" && (
        <div className="panel p-4">
          <div className="kicker mb-4">Normalized dimensions · {data.genome.algorithm_version}</div>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            {data.genome.dimensions.map((d) => (
              <div key={d.id} className="border border-line rounded-lg p-3">
                <div className="text-[11px] text-mute">{d.label}</div>
                <div className="font-mono text-xl mt-1">{d.value.toFixed(0)}</div>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === "Timeline" && (
        <ol className="space-y-3">
          {data.timeline.length === 0 ? (
            <div className="text-sm text-mute">No persisted events yet. Refresh to emit ScoreUpdated / LiquidityUpdated.</div>
          ) : (
            data.timeline.map((e) => (
              <li key={e.at + e.title} className="panel p-4 text-sm">
                <div className="font-mono text-[11px] text-mute">
                  {new Date(e.at).toLocaleString()} · {e.kind} · {e.source}
                </div>
                <div className="mt-1">{e.title}</div>
                {e.delta ? <div className="text-xs text-mute mt-1">Δ {e.delta}</div> : null}
              </li>
            ))
          )}
        </ol>
      )}

      {tab === "Baselines" && <TokenBaselines id={data.token.id} />}

      {tab === "Claims" && <TokenClaims id={data.token.id} />}

      {tab === "History" && <TokenHistory id={data.token.id} />}
    </div>
  );
}

function TokenOnchainIndex({ id }: { id: string }) {
  const [transfers, setTransfers] = useState<{ tx_hash: string; from: string; to: string; amount_raw: string }[]>([]);
  const [events, setEvents] = useState<{ direction: string; wallet?: string; exchange?: string; amount_usd?: number | null }[]>([]);
  const [clusters, setClusters] = useState<{ cluster_id: string; members: string[]; confidence: number }[]>([]);
  const [note, setNote] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.transfers(id), api.whaleEvents(id), api.clusters(id)])
      .then(([t, w, c]) => {
        setTransfers(t.transfers ?? []);
        setEvents(w.events ?? []);
        setClusters(c.clusters ?? []);
        if (t.note) setNote(t.note);
      })
      .catch((e) => setNote(String(e.message ?? e)));
  }, [id]);

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
      <div className="panel p-4">
        <div className="kicker mb-2">Indexed transfers</div>
        {note && transfers.length === 0 ? <div className="text-sm text-mute">{note}</div> : null}
        <ul className="text-xs font-mono space-y-1">
          {transfers.slice(0, 8).map((t) => (
            <li key={t.tx_hash + t.from + t.to}>
              {t.from.slice(0, 8)}… → {t.to.slice(0, 8)}… · {t.tx_hash.slice(0, 10)}…
            </li>
          ))}
        </ul>
      </div>
      <div className="panel p-4">
        <div className="kicker mb-2">Whale / CEX events</div>
        {events.length === 0 ? (
          <div className="text-sm text-mute">No classified movements in the indexed window.</div>
        ) : (
          <ul className="text-xs space-y-1">
            {events.slice(0, 8).map((e, i) => (
              <li key={`${e.direction}-${e.wallet}-${i}`}>
                {e.direction}
                {e.exchange ? ` · ${e.exchange}` : ""}
                {e.amount_usd != null ? ` · ~$${e.amount_usd.toFixed(0)}` : ""}
              </li>
            ))}
          </ul>
        )}
        {clusters.length > 0 ? (
          <div className="text-xs text-mute mt-3">
            {clusters.length} wallet cluster{clusters.length === 1 ? "" : "s"} (path ≠ identity)
          </div>
        ) : null}
      </div>
    </div>
  );
}

function TokenBaselines({ id }: { id: string }) {
  const [rows, setRows] = useState<
    {
      metric: string;
      window: string;
      n: number;
      mean: number;
      last: number;
      z_score: number | null;
      mad?: number;
      percentile_25?: number | null;
      percentile_75?: number | null;
      data_state: string;
    }[]
  >([]);
  const [pools, setPools] = useState<{ pair_address: string; dex?: string | null; liquidity_usd?: number | null }[]>([]);
  const [wallets, setWallets] = useState<{ address: string; classification: string; share_pct?: number | null }[]>([]);
  const [note, setNote] = useState<string | null>(null);

  useEffect(() => {
    Promise.all([api.baselines(id), api.pools(id), api.wallets(id)])
      .then(([b, p, w]) => {
        setRows(b.baselines ?? []);
        setPools(p.pools ?? []);
        setWallets(w.wallets ?? []);
        if (!b.baselines?.length) setNote("Baselines need persisted live history (Docker Postgres + a few refresh cycles).");
      })
      .catch((e) => setNote(String(e.message ?? e)));
  }, [id]);

  return (
    <div className="space-y-4">
      <p className="text-sm text-mute">
        Current vs historical baseline. Anomaly ≠ trade signal. Transfers stay blank until an RPC indexer cursor exists.
      </p>
      {note ? <div className="text-sm text-mute">{note}</div> : null}
      <div className="panel overflow-auto">
        <table className="w-full text-sm">
          <thead className="text-mute font-mono text-[11px] uppercase border-b border-line">
            <tr>
              {["Metric", "Window", "n", "Mean", "Last", "z", "MAD", "p25", "p75", "State"].map((h) => (
                <th key={h} className="text-left px-3 py-2 font-medium">{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={`${r.metric}-${r.window}`} className="border-b border-line/60">
                <td className="px-3 py-2 font-mono">{r.metric}</td>
                <td className="px-3 py-2">{r.window}</td>
                <td className="px-3 py-2 font-mono">{r.n}</td>
                <td className="px-3 py-2 font-mono">{r.mean.toExponential(2)}</td>
                <td className="px-3 py-2 font-mono">{r.last.toExponential(2)}</td>
                <td className="px-3 py-2 font-mono">{r.z_score != null ? r.z_score.toFixed(2) : "—"}</td>
                <td className="px-3 py-2 font-mono">{r.mad != null ? r.mad.toExponential(2) : "—"}</td>
                <td className="px-3 py-2 font-mono">{r.percentile_25 != null ? r.percentile_25.toExponential(2) : "—"}</td>
                <td className="px-3 py-2 font-mono">{r.percentile_75 != null ? r.percentile_75.toExponential(2) : "—"}</td>
                <td className="px-3 py-2"><DataStateBadge state={r.data_state} /></td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div className="panel p-4">
          <div className="kicker mb-2">Discovered pools</div>
          {pools.length === 0 ? (
            <div className="text-sm text-mute">No pair addresses from DexScreener yet.</div>
          ) : (
            <ul className="text-xs font-mono space-y-1">
              {pools.slice(0, 8).map((p) => (
                <li key={p.pair_address}>
                  {p.dex ?? "dex"} · {p.pair_address.slice(0, 10)}…{p.pair_address.slice(-4)}
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="panel p-4">
          <div className="kicker mb-2">Wallet classes (not identity)</div>
          {wallets.length === 0 ? (
            <div className="text-sm text-mute">Need Ethplorer top holders + Postgres.</div>
          ) : (
            <ul className="text-xs space-y-1">
              {wallets.slice(0, 8).map((w) => (
                <li key={w.address} className="font-mono">
                  {w.classification} · {(w.share_pct ?? 0).toFixed(2)}% · {w.address.slice(0, 8)}…
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>
    </div>
  );
}

function TokenClaims({ id }: { id: string }) {
  const [rows, setRows] = useState<
    { id: string; claim: string; relation: string; evidence: string[]; data_state: string; confidence: number; stored_at?: string | null }[]
  >([]);
  const [stored, setStored] = useState(0);
  const [note, setNote] = useState<string | null>(null);

  useEffect(() => {
    api
      .claims(id)
      .then((r) => {
        setRows(r.claims);
        setStored(r.persisted_count);
        setNote(r.note ?? null);
      })
      .catch((e) => setNote(String(e.message ?? e)));
  }, [id]);

  return (
    <div className="space-y-3">
      <p className="text-sm text-mute">
        Derived from the current snapshot. SUPPORTS is not SAFE. INSUFFICIENT is not a quiet market. VERIFIED ≠ SAFE.
        Persisted rows: {stored || "—"}.
      </p>
      {note ? <div className="text-xs text-mute">{note}</div> : null}
      <div className="space-y-2">
        {rows.length === 0 ? (
          <div className="text-sm text-mute">No claims derived for this snapshot.</div>
        ) : (
          rows.map((c) => (
            <div key={c.id} className="panel p-4">
              <div className="flex justify-between gap-3 items-start">
                <div>
                  <div className="font-mono text-[11px] text-mute">{c.id}</div>
                  <div className="text-sm mt-1">{c.claim}</div>
                </div>
                <span
                  className={`font-mono text-xs uppercase ${
                    c.relation === "supports"
                      ? "text-phosphor"
                      : c.relation === "contradicts"
                        ? "text-danger"
                        : "text-mute"
                  }`}
                >
                  {c.relation}
                </span>
              </div>
              <div className="mt-2 flex gap-2 items-center">
                <DataStateBadge state={c.data_state} />
                <span className="text-xs text-mute font-mono">conf {(c.confidence * 100).toFixed(0)}%</span>
              </div>
              <ul className="mt-2 text-xs text-mute font-mono space-y-0.5">
                {c.evidence.map((e) => (
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

function TokenHistory({ id }: { id: string }) {
  const [range, setRange] = useState("30d");
  const [rows, setRows] = useState<{ t: string; price_usd: number | null; health: number | null; data_state: string }[]>([]);
  const [note, setNote] = useState<string | null>(null);

  useEffect(() => {
    api
      .history(id, range)
      .then((r) => {
        setRows(r.points.map((p) => ({ t: p.t, price_usd: p.price_usd, health: p.health, data_state: p.data_state })));
        setNote(r.points.length === 0 ? "No persisted live snapshots in this range yet." : null);
      })
      .catch((e) => setNote(String(e.message ?? e)));
  }, [id, range]);

  return (
    <div className="space-y-4">
      <div className="flex gap-2 items-center">
        {["7d", "30d", "90d"].map((r) => (
          <button
            key={r}
            onClick={() => setRange(r)}
            className={`px-3 py-1 text-xs font-mono border rounded ${range === r ? "border-phosphor text-phosphor" : "border-line text-mute"}`}
          >
            {r}
          </button>
        ))}
        <a href={`/v1/tokens/${id}/history.csv?range=${range}`} className="ml-auto text-xs text-signal font-mono">
          CSV
        </a>
      </div>
      {note ? <div className="text-sm text-mute">{note}</div> : null}
      <div className="panel p-4 h-56">
        <ResponsiveContainer>
          <AreaChart data={rows.map((p) => ({ t: p.t, v: p.price_usd ?? 0 }))}>
            <XAxis dataKey="t" hide />
            <Tooltip contentStyle={{ background: "#0d1018", border: "1px solid #1c2333" }} />
            <Area type="monotone" dataKey="v" stroke="#4da6ff" fill="rgba(77,166,255,0.15)" />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
