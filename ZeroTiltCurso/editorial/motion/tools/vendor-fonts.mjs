// Copia para public/fonts/ as fontes OFL usadas nos vídeos (mesmos arquivos do Google Fonts
// que @remotion/google-fonts baixaria), para o render não depender de rede.
// Uso: node tools/vendor-fonts.mjs   (gera public/fonts/*.woff2 e src/fonts.json)
import { createHash } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const HERE = dirname(dirname(fileURLToPath(import.meta.url)));
const SUBSETS = ["latin", "latin-ext"];
const WANT = [
  { module: "Manrope", style: "normal", weights: ["400", "500", "600", "700", "800"] },
  { module: "Fraunces", style: "normal", weights: ["500", "600", "700"] },
  { module: "Fraunces", style: "italic", weights: ["600", "700"] },
];

const out = join(HERE, "public", "fonts");
mkdirSync(out, { recursive: true });
const faces = [];
for (const want of WANT) {
  const info = require(`@remotion/google-fonts/${want.module}`).getInfo();
  for (const weight of want.weights) {
    for (const subset of SUBSETS) {
      const url = info.fonts[want.style][weight][subset];
      const response = await fetch(url);
      if (!response.ok) throw new Error(`${response.status} ${url}`);
      const bytes = Buffer.from(await response.arrayBuffer());
      const file = `${want.module.toLowerCase()}-${want.style}-${weight}-${subset}.woff2`;
      writeFileSync(join(out, file), bytes);
      faces.push({
        family: info.fontFamily,
        style: want.style,
        weight,
        unicodeRange: info.unicodeRanges[subset],
        file: `fonts/${file}`,
        source: url,
        sha256: createHash("sha256").update(bytes).digest("hex"),
      });
    }
  }
}
writeFileSync(join(HERE, "src", "fonts.json"), JSON.stringify(faces, null, 1) + "\n");
console.log(`${faces.length} arquivos em public/fonts`);
