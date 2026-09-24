/** Só http(s). Bloqueia javascript:, data:, //host sem esquema. */
export function safeHttpUrl(raw: string | undefined | null): string | null {
  if (!raw) return null;
  const trimmed = raw.trim();
  if (!/^https?:\/\//i.test(trimmed)) return null;
  try {
    const url = new URL(trimmed);
    if (url.protocol !== "http:" && url.protocol !== "https:") return null;
    return url.href;
  } catch {
    return null;
  }
}

/** Aceita somente caminhos da própria SPA e rejeita formas ambíguas de redirecionamento. */
export function safeInternalPath(
  raw: string | undefined | null,
  fallback: string,
): string {
  if (!raw) return fallback;
  const trimmed = raw.trim();
  const hasControlCharacter = [...trimmed].some((character) => {
    const code = character.charCodeAt(0);
    return code <= 0x1f || code === 0x7f;
  });
  if (
    !trimmed.startsWith("/") ||
    trimmed.startsWith("//") ||
    trimmed.includes("\\") ||
    /%5c/i.test(trimmed) ||
    hasControlCharacter
  ) {
    return fallback;
  }

  try {
    const base = new URL("https://zerotilt.invalid");
    const target = new URL(trimmed, base);
    if (target.origin !== base.origin) return fallback;
    return `${target.pathname}${target.search}${target.hash}`;
  } catch {
    return fallback;
  }
}
