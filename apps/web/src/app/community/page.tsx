"use client";

import { api, eventsUrl } from "@/lib/api";
import { DataStateBadge, ErrorState } from "@/components/ui";
import Link from "next/link";
import { useEffect, useRef, useState } from "react";

type Row = {
  id: string;
  symbol: string;
  name: string;
  website?: string | null;
  socials: {
    twitter?: string | null;
    telegram?: string | null;
    github?: string | null;
    discord?: string | null;
  };
  social_state: string;
  mentions_24h?: number | string | null;
  chat_state?: string;
  chat_messages?: number;
  note?: string;
};

type Msg = { id: string; handle: string; body: string; created_at: string };

const HANDLE_KEY = "mcos.chatHandle";
const SESSION_KEY = "mcos.chatSession";

export default function CommunityPage() {
  const [rows, setRows] = useState<Row[]>([]);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [discordOn, setDiscordOn] = useState(false);
  const [telegramOn, setTelegramOn] = useState(false);
  const [connectNote, setConnectNote] = useState("");
  const [rooms, setRooms] = useState<{ id: string; label: string; messages?: number }[]>([]);
  const [room, setRoom] = useState("global");
  const [messages, setMessages] = useState<Msg[]>([]);
  const [handle, setHandle] = useState("");
  const [session, setSession] = useState("");
  const [draft, setDraft] = useState("");
  const [chatNote, setChatNote] = useState<string | null>(null);
  const [connected, setConnected] = useState<string | null>(null);
  const [connectReason, setConnectReason] = useState<string | null>(null);
  const [chatLive, setChatLive] = useState(true);
  const [hydrated, setHydrated] = useState(false);
  const bottom = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (typeof window === "undefined") return;
    const q = new URLSearchParams(window.location.search);
    setConnected(q.get("connected") ?? q.get("connect"));
    setConnectReason(q.get("reason"));
    setHandle(window.localStorage.getItem(HANDLE_KEY) ?? "");
    setSession(window.localStorage.getItem(SESSION_KEY) ?? "");
    const roomQ = q.get("room");
    if (roomQ) setRoom(roomQ);
    setHydrated(true);
  }, []);

  useEffect(() => {
    if (!hydrated || session) return;
    api
      .communitySession(handle || "")
      .then((s) => {
        window.localStorage.setItem(HANDLE_KEY, s.handle);
        window.localStorage.setItem(SESSION_KEY, s.session_id);
        setSession(s.session_id);
        setHandle(s.handle);
      })
      .catch((e) => setChatNote(String((e as Error).message ?? e)));
  }, [hydrated, session, handle]);

  useEffect(() => {
    Promise.all([
      api.community(),
      api.connectStatus().catch(() => null),
      api.communityRooms().catch(() => ({ rooms: [], live: true })),
    ])
      .then(([d, c, r]) => {
        setRows(d.tokens ?? []);
        setNote(d.note ?? null);
        setChatLive(d.chat?.live !== false);
        if (c) {
          setDiscordOn(c.discord.configured);
          setTelegramOn(c.telegram.configured);
          setConnectNote(`${c.discord.note} ${c.telegram.note}`);
        }
        setRooms(r.rooms ?? []);
      })
      .catch((e) => setError(String(e.message ?? e)));
  }, []);

  useEffect(() => {
    let stop = false;
    async function load() {
      try {
        const d = await api.communityMessages(room);
        if (!stop) setMessages(d.messages ?? []);
      } catch {
        if (!stop) setMessages([]);
      }
    }
    void load();
    const tick = window.setInterval(() => void load(), 4000);
    return () => {
      stop = true;
      window.clearInterval(tick);
    };
  }, [room]);

  useEffect(() => {
    const es = new EventSource(eventsUrl());
    es.onmessage = (ev) => {
      try {
        const p = JSON.parse(ev.data) as { type?: string; room_id?: string; message?: Msg };
        if (p.type === "community.message" && p.room_id === room && p.message) {
          setMessages((prev) => (prev.some((m) => m.id === p.message!.id) ? prev : [...prev, p.message!]));
        }
      } catch {
        /* ignore */
      }
    };
    return () => es.close();
  }, [room]);

  useEffect(() => {
    bottom.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length]);

  async function send() {
    setChatNote(null);
    try {
      const m = await api.communityPost(room, draft, session);
      if (m.session_id) {
        window.localStorage.setItem(SESSION_KEY, m.session_id);
        setSession(m.session_id);
      }
      setDraft("");
      setMessages((prev) => (prev.some((x) => x.id === m.id) ? prev : [...prev, m]));
    } catch (e) {
      setChatNote(String((e as Error).message ?? e));
    }
  }

  async function openTelegram() {
    try {
      const t = await api.telegramStart();
      window.open(t.url, "_blank", "noopener,noreferrer");
    } catch (e) {
      setChatNote(String((e as Error).message ?? e));
    }
  }

  if (error) return <ErrorState message={error} />;

  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <div className="kicker">Community desk</div>
        <h1 className="text-3xl mt-1">Chat ao vivo</h1>
        <p className="text-sm text-mute mt-2">
          O chat da plataforma está <span className="text-phosphor">LIVE</span>. Qualquer pessoa
          escreve sobre memes e projetos. Menções X/Telegram de firehose ficam não ligadas sem
          licença — isso não desliga este chat.
        </p>
      </div>

      {connected ? (
        <p className="text-sm text-phosphor">
          Connect result: {connected}
          {connectReason ? <span className="text-mute"> · reason={connectReason}</span> : null}
        </p>
      ) : null}

      <div className="panel p-4 space-y-3">
        <div className="flex justify-between items-center">
          <div className="kicker">Live chat</div>
          <span className="font-mono text-[10px] uppercase text-phosphor">{chatLive ? "live" : "offline"}</span>
        </div>
        <select
          className="bg-void border border-line rounded-lg px-3 py-2 text-sm"
          value={room}
          onChange={(e) => setRoom(e.target.value)}
        >
          {rooms.map((r) => (
            <option key={r.id} value={r.id}>
              {r.label}
              {typeof r.messages === "number" ? ` · ${r.messages}` : ""}
            </option>
          ))}
        </select>
        <div className="h-80 overflow-y-auto border border-line rounded-lg p-3 space-y-2 bg-[#070910]">
          {messages.length === 0 ? (
            <p className="text-sm text-mute">Sala vazia e operacional. A primeira mensagem fica aqui.</p>
          ) : (
            messages.map((m) => (
              <div key={m.id} className="text-sm">
                <span className="font-mono text-[11px] text-phosphor">{m.handle}</span>{" "}
                <span>{m.body}</span>
              </div>
            ))
          )}
          <div ref={bottom} />
        </div>
        <div className="flex flex-col sm:flex-row gap-2">
          <input
            className="sm:w-40 bg-void border border-line rounded-lg px-3 py-2 text-sm font-mono"
            placeholder="handle"
            value={handle}
            readOnly
          />
          <input
            className="flex-1 bg-void border border-line rounded-lg px-3 py-2 text-sm"
            placeholder="Escreve sobre um meme ou projeto…"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void send();
            }}
          />
          <button className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm" type="button" onClick={send}>
            Enviar
          </button>
        </div>
        {chatNote ? <p className="text-xs text-mute">{chatNote}</p> : null}
      </div>

      <div className="panel p-4 space-y-3">
        <div className="kicker">Alertas no teu Discord / Telegram</div>
        <p className="text-xs text-mute">
          Isto é ligação da tua sala para receber alertas. Não substitui o chat acima. Sem app/bot no
          servidor, os botões ficam à espera — os links oficiais do projeto continuam abaixo.
        </p>
        <div className="flex flex-wrap gap-2">
          {discordOn ? (
            <a className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm" href="/v1/connect/discord">
              Ligar Discord
            </a>
          ) : (
            <span className="text-sm text-mute">Discord OAuth: DISCORD_CLIENT_ID / SECRET no Cloud Run.</span>
          )}
          {telegramOn ? (
            <button className="bg-phosphor text-void rounded-lg px-4 py-2 text-sm" type="button" onClick={openTelegram}>
              Ligar Telegram
            </button>
          ) : (
            <span className="text-sm text-mute">Telegram bot: TELEGRAM_BOT_TOKEN / USERNAME no Cloud Run.</span>
          )}
        </div>
        <p className="text-[11px] text-mute">{connectNote}</p>
      </div>

      {note ? <p className="text-xs text-mute">{note}</p> : null}
      <div className="space-y-3">
        {rows.map((t) => {
          const links = [
            ["Website", t.website],
            ["X / Twitter", t.socials.twitter],
            ["Telegram", t.socials.telegram],
            ["Discord", t.socials.discord],
            ["GitHub", t.socials.github],
          ].filter(([, href]) => !!href) as [string, string][];
          return (
            <section key={t.id} className="panel p-4 space-y-2">
              <div className="flex justify-between gap-2">
                <div>
                  <div className="text-lg">{t.name}</div>
                  <div className="font-mono text-xs text-mute">{t.symbol}</div>
                </div>
                <div className="flex gap-2 items-center">
                  <span className="font-mono text-[10px] uppercase text-phosphor">chat live</span>
                  <DataStateBadge state={t.social_state} />
                </div>
              </div>
              {links.length === 0 ? (
                <p className="text-sm text-mute">No official social URL in the registry.</p>
              ) : (
                <ul className="text-sm space-y-1">
                  {links.map(([label, href]) => (
                    <li key={label}>
                      <a className="text-phosphor underline" href={href} target="_blank" rel="noreferrer">
                        {label}
                      </a>
                    </li>
                  ))}
                </ul>
              )}
              <p className="text-xs text-mute">
                Chat OS: {t.chat_messages ?? 0} · Firehose 24h: {t.mentions_24h ?? "not connected"} · {t.note}
              </p>
              <div className="flex gap-3 text-xs">
                <Link className="text-phosphor underline" href={`/twin/${t.id}`}>
                  Open Twin
                </Link>
                <button className="text-phosphor underline" type="button" onClick={() => setRoom(t.id)}>
                  Abrir sala
                </button>
              </div>
            </section>
          );
        })}
      </div>
    </div>
  );
}
