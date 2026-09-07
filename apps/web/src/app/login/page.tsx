"use client";

import { api } from "@/lib/api";
import Link from "next/link";
import { useEffect, useState } from "react";

const SESSION_KEY = "mcos.oidcSession";
const EMAIL_KEY = "mcos.oidcEmail";

export default function LoginPage() {
  const [configured, setConfigured] = useState<boolean | null>(null);
  const [issuer, setIssuer] = useState<string | null>(null);
  const [note, setNote] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [email, setEmail] = useState<string | null>(null);
  const [sessionSaved, setSessionSaved] = useState(false);

  useEffect(() => {
    api
      .sso()
      .then((s) => {
        setConfigured(Boolean(s.configured));
        setIssuer(s.issuer ?? null);
        setNote(s.note ?? null);
      })
      .catch(() => {
        setConfigured(false);
        setNote("SSO status unavailable.");
      });

    if (typeof window === "undefined") return;
    const q = new URLSearchParams(window.location.search);
    const err = q.get("error");
    if (err) setError(err);
    const sess = q.get("session");
    const em = q.get("email");
    if (sess && q.get("ok") === "1") {
      window.sessionStorage.setItem(SESSION_KEY, sess);
      if (em) window.sessionStorage.setItem(EMAIL_KEY, em);
      setEmail(em);
      setSessionSaved(true);
      window.history.replaceState({}, "", "/login/");
    } else {
      setEmail(window.sessionStorage.getItem(EMAIL_KEY));
      setSessionSaved(Boolean(window.sessionStorage.getItem(SESSION_KEY)));
    }
  }, []);

  const google =
    configured && issuer && issuer.includes("accounts.google.com");

  return (
    <div className="space-y-6 max-w-lg">
      <div>
        <div className="kicker">Account</div>
        <h1 className="text-3xl mt-1">Sign in</h1>
        <p className="text-sm text-mute mt-2">
          Identity from your IdP only. No local passwords. No fake users. Paying never changes a score.
          EXECUTE stays off.
        </p>
      </div>

      {sessionSaved ? (
        <div className="panel p-5 space-y-2">
          <div className="text-phosphor text-sm">Signed in{email ? ` · ${email}` : ""}</div>
          <p className="text-xs text-mute">
            Session kept in this browser tab only (sessionStorage). Twin facts stay shared public
            intelligence.
          </p>
          <Link href="/" className="text-sm text-phosphor underline">
            Back to Overview
          </Link>
        </div>
      ) : null}

      {error ? (
        <div className="panel p-4 text-sm text-danger">Sign-in failed · reason={error}</div>
      ) : null}

      <div className="panel p-5 space-y-4">
        {configured === null ? (
          <div className="text-sm text-mute">Checking SSO…</div>
        ) : configured ? (
          <>
            <p className="text-sm text-mute">
              Provider: <span className="font-mono text-xs">{issuer}</span>
            </p>
            <a
              className="inline-flex items-center justify-center bg-phosphor text-void rounded-lg px-4 py-2 text-sm font-medium"
              href="/v1/auth/oidc/login"
            >
              {google ? "Continue with Google" : "Continue with IdP"}
            </a>
          </>
        ) : (
          <div className="space-y-2 text-sm text-mute">
            <p>Google / OIDC is not connected on this API yet.</p>
            <p>
              Operator: set <code className="font-mono text-xs">OIDC_ISSUER=https://accounts.google.com</code>,{" "}
              <code className="font-mono text-xs">OIDC_CLIENT_ID</code>,{" "}
              <code className="font-mono text-xs">OIDC_CLIENT_SECRET</code> on Cloud Run. Redirect URI must be{" "}
              <code className="font-mono text-xs">
                https://memecoin-os.web.app/v1/auth/oidc/callback
              </code>
              .
            </p>
            {note ? <p className="text-xs">{note}</p> : null}
          </div>
        )}
      </div>

      <p className="text-xs text-mute">
        <Link href="/terms/" className="underline text-phosphor">
          Terms
        </Link>
        {" · "}
        <Link href="/privacy/" className="underline text-phosphor">
          Privacy
        </Link>
      </p>
    </div>
  );
}
