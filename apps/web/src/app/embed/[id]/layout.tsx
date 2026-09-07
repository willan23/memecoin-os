import { liveTokenParams } from "@/lib/static-token-ids";

export async function generateStaticParams() {
  return liveTokenParams();
}

export default function EmbedLayout({ children }: { children: React.ReactNode }) {
  return children;
}
