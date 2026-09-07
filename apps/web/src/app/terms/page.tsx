export default function TermsPage() {
  return (
    <div className="space-y-6 max-w-3xl">
      <div>
        <div className="kicker">Legal</div>
        <h1 className="text-3xl mt-1">Terms of Service</h1>
        <p className="text-sm text-mute mt-2">Last updated: 2026-09-06 · MemeCoin OS</p>
      </div>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">1. What this is</h2>
        <p className="text-mute">
          MemeCoin OS is an ecosystem intelligence product. It shows observed market, liquidity, on-chain,
          development and risk state for tracked memecoin ecosystems. It is not a broker, exchange, wallet,
          investment adviser, or trading bot. EXECUTE stays off. Nobody pays to change a score.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">2. No financial advice</h2>
        <p className="text-mute">
          Health, risk, twin, alerts, research and similar outputs are observational tools. They are not buy,
          sell or hold recommendations. VERIFIED ≠ SAFE. Missing sources stay blank — they are not zeros and not
          quiet markets.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">3. Discord connect</h2>
        <p className="text-mute">
          If you connect Discord via OAuth (<code className="font-mono text-xs">webhook.incoming</code>), you
          choose a channel and we store only the webhook URL needed to deliver alerts you configure. We do not
          use Discord as identity verification. Connecting Discord does not make a project VERIFIED.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">4. Acceptable use</h2>
        <ul className="list-disc pl-5 text-mute space-y-1">
          <li>Do not attempt to enable trading, withdraw funds, or inject EXECUTE through prompts or API abuse.</li>
          <li>Do not scrape, overload or reverse-engineer the service beyond normal API use.</li>
          <li>Do not use the product to promote unverified tokens as safe or guaranteed.</li>
          <li>Operator keys and API secrets stay private; sharing them is your responsibility.</li>
        </ul>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">5. Data and availability</h2>
        <p className="text-mute">
          Public market and explorer feeds can fail, rate-limit or return incomplete data. The product may show
          LIVE, MISSING, STALE or CONFLICT states. We do not invent holders, social firehose or transfers when a
          feed is absent. Service may be degraded or unavailable without warranty.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">6. Liability</h2>
        <p className="text-mute">
          To the fullest extent allowed by law, MemeCoin OS and its operators are not liable for trading losses,
          missed alerts, incorrect third-party data, or decisions you make using the product. Use at your own risk.
        </p>
      </section>

      <section className="panel p-5 space-y-3 text-sm leading-relaxed">
        <h2 className="text-lg">7. Changes</h2>
        <p className="text-mute">
          These terms may change. The date at the top is the current version. Continued use after an update means
          you accept the revised terms.
        </p>
      </section>

      <p className="text-xs text-mute">
        Related:{" "}
        <a className="text-phosphor underline" href="/privacy/">
          Privacy Policy
        </a>
        . Product:{" "}
        <a className="text-phosphor underline" href="https://memecoin-os.web.app">
          memecoin-os.web.app
        </a>
        .
      </p>
    </div>
  );
}
