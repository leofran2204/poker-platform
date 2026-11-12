import { cancelRender, continueRender, delayRender, staticFile } from "remotion";
import faces from "./fonts.json";

// Fontes abertas (OFL, licenças em public/fonts) copiadas por tools/vendor-fonts.mjs: os mesmos
// arquivos do Google Fonts, carregados do projeto para o render não depender de rede.
// Fraunces inclui o itálico verdadeiro (600 e 700) usado nos destaques dos títulos.
const waiting = delayRender("Carregando fontes locais");
Promise.all(
  faces.map(async (face) => {
    const font = new FontFace(face.family, `url(${staticFile(face.file)}) format("woff2")`, {
      weight: face.weight,
      style: face.style,
      unicodeRange: face.unicodeRange,
    });
    // FontFaceSet.add falta na lib DOM desta versão do TypeScript.
    (document.fonts as unknown as { add(face: FontFace): void }).add(await font.load());
  }),
)
  .then(() => continueRender(waiting))
  .catch((error) => cancelRender(error));

export const UI = "Manrope";
export const DISPLAY = "Fraunces";

// Paleta derivada do tema do site (felt/gold em Frontend-Web/src/index.css) e da marca.
export const C = {
  bg0: "#050e09",
  bg1: "#0b2116",
  felt0: "#23744a",
  felt1: "#155232",
  felt2: "#0b3420",
  rail0: "#3a2618",
  rail1: "#140c07",
  gold: "#d4a843",
  goldSoft: "#ecd99a",
  goldDim: "#8b6914",
  ivory: "#f7f1e3",
  paper: "#fbf8f0",
  ink: "#101010",
  muted: "#a9bfb2",
  faint: "rgba(233, 240, 235, 0.55)",
  mint: "#6fd0ae",
  red: "#c8102e",
  danger: "#ef6a5f",
  ok: "#5fd394",
  panel: "rgba(7, 20, 13, 0.82)",
  line: "rgba(236, 217, 154, 0.22)",
} as const;

export const W = 1920;
export const H = 1080;
