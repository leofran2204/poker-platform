// Sons da mesa (os mesmos efeitos dos vídeos da Academy, em public/sounds).
// O navegador só libera áudio depois de um gesto do jogador; até lá, tocar falha
// em silêncio. A preferência de som fica guardada neste navegador.

export type TableSound = "chip" | "chips" | "deal" | "flip" | "pop";

const STORAGE_KEY = "zt-table-sound";
const VOLUME: Record<TableSound, number> = { chip: 0.5, chips: 0.5, deal: 0.45, flip: 0.5, pop: 0.6 };
const cache = new Map<TableSound, HTMLAudioElement>();

export function soundEnabled(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) !== "off";
  } catch {
    return true;
  }
}

export function setSoundEnabled(enabled: boolean): void {
  try {
    localStorage.setItem(STORAGE_KEY, enabled ? "on" : "off");
  } catch {
    // Sem armazenamento (navegação privada): vale só para esta sessão.
  }
}

export function playTableSound(name: TableSound): void {
  if (typeof Audio === "undefined" || !soundEnabled()) return;
  try {
    let base = cache.get(name);
    if (!base) {
      base = new Audio(`/sounds/${name}.wav`);
      base.preload = "auto";
      cache.set(name, base);
    }
    // Clona para permitir sons sobrepostos (várias fichas em sequência).
    const voice = base.cloneNode(true) as HTMLAudioElement;
    voice.volume = VOLUME[name];
    void voice.play().catch(() => {});
  } catch {
    // Áudio indisponível: a mesa continua funcionando sem som.
  }
}

/** Vibração curta no celular quando chega a vez do jogador. */
export function buzz(): void {
  if (!soundEnabled()) return;
  try {
    navigator.vibrate?.(80);
  } catch {
    // Sem suporte a vibração.
  }
}
