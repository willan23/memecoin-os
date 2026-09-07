import { liveTokenParams } from "@/lib/static-token-ids";

export async function generateStaticParams() {
  return liveTokenParams();
}

export default function TwinLayout({ children }: { children: React.ReactNode }) {
  return children;
}
