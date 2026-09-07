const SEEDS = ["pepe", "kishu-inu", "baby-doge", "arbdoge-ai"];

/** Used at `next export` so Hosting has a real HTML file per live token. */
export async function liveTokenParams(): Promise<{ id: string }[]> {
  const base = (process.env.NEXT_PUBLIC_API_URL || "https://memecoin-os.web.app").replace(/\/$/, "");
  const ids = new Set(SEEDS);
  try {
    const res = await fetch(`${base}/v1/overview`, { cache: "no-store" });
    if (res.ok) {
      const data = (await res.json()) as {
        tokens?: { token?: { id?: string } }[];
      };
      for (const row of data.tokens ?? []) {
        const id = row.token?.id;
        if (id) ids.add(id);
      }
    }
  } catch {
    // Seeds only — export must not fail if the API is cold.
  }
  return [...ids].sort().map((id) => ({ id }));
}
