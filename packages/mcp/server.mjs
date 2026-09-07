#!/usr/bin/env node
/**
 * Stdio MCP client for MemeCoin OS.
 * Forwards JSON-RPC to the live Exchange Intelligence API.
 * EXECUTE is never exposed. Missing sources stay blank.
 */
import { createInterface } from "node:readline";

const API = (process.env.MEMECOIN_OS_API || "https://memecoin-os-api-763598651987.europe-west1.run.app").replace(
  /\/$/,
  ""
);

async function rpc(body) {
  const res = await fetch(`${API}/v1/mcp`, {
    method: "POST",
    headers: { "content-type": "application/json", accept: "application/json" },
    body: JSON.stringify(body),
  });
  const text = await res.text();
  try {
    return JSON.parse(text);
  } catch {
    return {
      jsonrpc: "2.0",
      id: body.id ?? null,
      error: { code: -32000, message: `upstream ${res.status}` },
    };
  }
}

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on("line", async (line) => {
  const raw = line.trim();
  if (!raw) return;
  let msg;
  try {
    msg = JSON.parse(raw);
  } catch {
    process.stdout.write(
      `${JSON.stringify({ jsonrpc: "2.0", id: null, error: { code: -32700, message: "parse error" } })}\n`
    );
    return;
  }
  if (typeof msg.method === "string" && msg.method.startsWith("notifications/")) {
    return;
  }
  const out = await rpc(msg);
  process.stdout.write(`${JSON.stringify(out)}\n`);
});
