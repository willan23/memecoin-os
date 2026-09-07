import type { NextConfig } from "next";

const API = process.env.NEXT_PUBLIC_API_URL ?? "http://127.0.0.1:8080";
const hosting = process.env.FIREBASE_HOSTING === "1";

const nextConfig: NextConfig = {
  ...(hosting
    ? {
        output: "export" as const,
        images: { unoptimized: true },
        trailingSlash: true,
      }
    : {}),
  ...(hosting
    ? {}
    : {
        async rewrites() {
          return [
            { source: "/v1/:path*", destination: `${API}/v1/:path*` },
            { source: "/v2/:path*", destination: `${API}/v2/:path*` },
            { source: "/health", destination: `${API}/health` },
          ];
        },
      }),
};

export default nextConfig;
