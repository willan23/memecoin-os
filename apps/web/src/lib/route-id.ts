"use client";

import { useParams } from "next/navigation";
import { useEffect, useState } from "react";

const PREFIXES = new Set(["tokens", "twin", "embed", "projects"]);

/** Hosting rewrites unknown /tokens/:id to the PEPE shell. Read the real id from the address bar. */
export function idFromPathname(pathname: string): string | undefined {
  const parts = pathname.split("/").filter(Boolean);
  if (parts.length >= 2 && PREFIXES.has(parts[0])) {
    return decodeURIComponent(parts[1]);
  }
  return undefined;
}

export function useEntityId(): string {
  const params = useParams<{ id: string }>();
  const fromParams = typeof params.id === "string" ? params.id : "";
  const [id, setId] = useState("");

  useEffect(() => {
    setId(idFromPathname(window.location.pathname) ?? fromParams);
  }, [fromParams]);

  return id;
}
