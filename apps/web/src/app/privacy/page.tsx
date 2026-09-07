export default function PrivacyPage() {
  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <div className="kicker">Legal</div>
        <h1 className="text-3xl mt-1">Privacy Policy</h1>
        <p className="text-sm text-mute mt-2">Last updated: 2026-09-06 · MemeCoin OS</p>
      </div>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">1. Scope</h2>
        <p className="text-mute">
          This policy describes what MemeCoin OS collects when you use{" "}
          <a className="text-phosphor underline" href="https://memecoin-os.web.app">
            memecoin-os.web.app
          </a>{" "}
          and related API routes. It is not legal advice. Token market intelligence on the platform is compiled
          from public sources and is not personal data about you.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">2. What we collect</h2>
        <ul className="list-disc pl-5 text-mute space-y-1">
          <li>
            <span className="text-ink">Operator / API keys</span> — hashed secrets for optional auth. Plaintext is
            shown once at create time and not stored.
          </li>
          <li>
            <span className="text-ink">Usage meters</span> — plan counters (API / research / AI) when metering is on.
          </li>
          <li>
            <span className="text-ink">Community chat</span> — guest session id, display handle and messages you
            post in first-party rooms. A handle is not verified identity.
          </li>
          <li>
            <span className="text-ink">Discord connect</span> — OAuth code exchange; we store the webhook URL you
            authorize for alerts. We do not store your Discord password. Scope used:{" "}
            <code className="font-mono text-xs">webhook.incoming</code>.
          </li>
          <li>
            <span className="text-ink">Telegram connect</span> — when configured, bot chat linkage for alert
            delivery you start.
          </li>
          <li>
            <span className="text-ink">Project claims / official links</span> — https URLs and claim labels you
            submit (operator key when required). Claim ≠ VERIFIED.
          </li>
          <li>
            <span className="text-ink">Technical logs</span> — request health, provider errors, job status. Not used
            to invent market facts.
          </li>
        </ul>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">3. What we do not collect</h2>
        <ul className="list-disc pl-5 text-mute space-y-1">
          <li>Private keys, seed phrases or wallet signing credentials.</li>
          <li>Payment card numbers in the product UI (Stripe Checkout only if you enable it later via secrets).</li>
          <li>A licensed social mention firehose — that plane stays not connected without a licence.</li>
        </ul>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">4. Why we process it</h2>
        <p className="text-mute">
          To run the intelligence product, deliver the alerts you connect, meter plan usage, and keep the service
          secure. Public token snapshots stay shared intelligence across tenants; isolation is keys, usage and
          queue — not a forked Twin.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">5. Third parties</h2>
        <p className="text-mute">
          Market and chain data may come from providers such as CoinGecko, DexScreener, Ethplorer, Etherscan and
          Solana RPC when configured. Hosting and API run on Firebase Hosting and Google Cloud Run. Discord or
          Telegram receive alert posts only after you connect them.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">6. Retention and control</h2>
        <p className="text-mute">
          Webhooks and chat messages persist until you remove them or an operator deletes them. You can stop
          Discord alerts by deleting the webhook in Discord or asking an operator to remove the stored URL. Revoke
          API keys in Settings. Browser session operator keys stay in that browser only.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">7. Contact</h2>
        <p className="text-mute">
          For privacy questions about this product, use the operator channel that manages the{" "}
          <code className="font-mono text-xs">memecoin-os</code> deployment. Do not paste secrets into chat
          assistants or public issues.
        </p>
      </section>

      <p className="text-xs text-mute">
        Related:{" "}
        <a className="text-phosphor underline" href="/terms/">
          Terms of Service
        </a>
        .
      </p>
    </div>
  );
}
