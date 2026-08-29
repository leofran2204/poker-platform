import { useEffect } from "react";

/** Marks the current route as noindex while mounted (SPA). */
export function NoIndex() {
  useEffect(() => {
    const existing = document.querySelector('meta[name="robots"]');
    const prev = existing?.getAttribute("content") ?? null;
    let meta = existing as HTMLMetaElement | null;
    if (!meta) {
      meta = document.createElement("meta");
      meta.name = "robots";
      document.head.appendChild(meta);
    }
    meta.content = "noindex, nofollow, noarchive";
    return () => {
      if (!meta) return;
      if (prev == null) {
        meta.remove();
      } else {
        meta.content = prev;
      }
    };
  }, []);
  return null;
}
