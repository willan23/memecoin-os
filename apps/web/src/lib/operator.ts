const STORAGE = "mcos.operatorKey";

export function operatorKey(): string {
  if (typeof window === "undefined") return "";
  return window.sessionStorage.getItem(STORAGE)?.trim() ?? "";
}

export function setOperatorKey(value: string) {
  if (typeof window === "undefined") return;
  const v = value.trim();
  if (v) window.sessionStorage.setItem(STORAGE, v);
  else window.sessionStorage.removeItem(STORAGE);
}

export function operatorHeaders(): Record<string, string> {
  const key = operatorKey();
  return key ? { "x-operator-key": key } : {};
}
