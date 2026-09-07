"""Create MemeCoin OS Stripe Products + monthly EUR Prices. Reads STRIPE_SECRET_KEY from .env.
Never prints the secret. Writes price ids back into .env (STRIPE_PRICE_*).
"""
from __future__ import annotations

import json
import ssl
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ENV = ROOT / ".env"

PLANS = [
    ("STRIPE_PRICE_PRO", "pro", "MemeCoin OS Pro", 19900),
    ("STRIPE_PRICE_RESEARCH", "research", "MemeCoin OS Research", 29900),
    ("STRIPE_PRICE_GROWTH", "growth", "MemeCoin OS Growth", 39900),
    ("STRIPE_PRICE_API", "api", "MemeCoin OS API", 49900),
    ("STRIPE_PRICE_ENTERPRISE", "enterprise", "MemeCoin OS Enterprise", 99900),
]


def load_env() -> dict[str, str]:
    out: dict[str, str] = {}
    for line in ENV.read_text(encoding="utf-8-sig").splitlines():
        if not line.strip() or line.strip().startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        out[k.strip()] = v.strip().strip('"').strip("'")
    return out


def stripe(secret: str, method: str, path: str, form: dict[str, str] | None = None) -> dict:
    data = urllib.parse.urlencode(form).encode() if form is not None else None
    req = urllib.request.Request(
        f"https://api.stripe.com/v1/{path}",
        data=data,
        method=method,
        headers={"Authorization": f"Bearer {secret}"},
    )
    if data is not None:
        req.add_header("Content-Type", "application/x-www-form-urlencoded")
    ctx = ssl.create_default_context()
    try:
        with urllib.request.urlopen(req, context=ctx, timeout=30) as resp:
            return json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8", "replace")
        raise SystemExit(f"stripe HTTP {e.code} {path}: {body[:300]}") from e


def upsert_env(updates: dict[str, str]) -> None:
    lines = ENV.read_text(encoding="utf-8-sig").splitlines()
    keys = set(updates)
    out: list[str] = []
    seen: set[str] = set()
    for line in lines:
        if "=" in line and not line.strip().startswith("#"):
            k = line.split("=", 1)[0].strip()
            if k in updates:
                out.append(f"{k}={updates[k]}")
                seen.add(k)
                continue
        out.append(line)
    for k, v in updates.items():
        if k not in seen:
            out.append(f"{k}={v}")
    ENV.write_text("\n".join(out) + "\n", encoding="utf-8")


def main() -> None:
    env = load_env()
    secret = env.get("STRIPE_SECRET_KEY", "")
    if not secret.startswith("sk_"):
        raise SystemExit("STRIPE_SECRET_KEY missing or not sk_… in .env")

    # Probe account without printing key
    acct = stripe(secret, "GET", "account")
    print("stripe_account", acct.get("id"), "livemode", acct.get("charges_enabled"))

    product = stripe(
        secret,
        "POST",
        "products",
        {
            "name": "MemeCoin OS",
            "description": "Ecosystem intelligence plans. Paying never changes a score. EXECUTE off.",
            "metadata[app]": "memecoin-os",
        },
    )
    product_id = product["id"]
    print("product", product_id)

    price_updates: dict[str, str] = {}
    for env_key, plan, nickname, amount in PLANS:
        existing = env.get(env_key, "")
        if existing.startswith("price_") and existing[6:].isalnum():
            print(env_key, "keep", existing)
            continue
        if existing:
            print(env_key, "replace_invalid_len", len(existing))
        price = stripe(
            secret,
            "POST",
            "prices",
            {
                "product": product_id,
                "currency": "eur",
                "unit_amount": str(amount),
                "recurring[interval]": "month",
                "nickname": nickname,
                "metadata[plan]": plan,
                "metadata[app]": "memecoin-os",
            },
        )
        price_updates[env_key] = price["id"]
        print(env_key, "created", price["id"], f"EUR {amount/100:.0f}/mo")

    if price_updates:
        upsert_env(price_updates)
        print("wrote", len(price_updates), "price ids into .env")
    else:
        print("no new prices (all STRIPE_PRICE_* already price_…)")


if __name__ == "__main__":
    main()
