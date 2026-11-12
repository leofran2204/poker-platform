import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { MemoryRouter } from "react-router";
import { describe, expect, it } from "vitest";
import { TermsPage } from "./TermsPage";

// O texto legal tem um único dono: Documentacao/TERMOS_DE_USO_E_SERVICO.md.
// Este teste falha se a página do site divergir dele.
const terms = readFileSync(
  fileURLToPath(new URL("../../../Documentacao/TERMOS_DE_USO_E_SERVICO.md", import.meta.url)),
  "utf8",
);
const lines = terms.split(/\r?\n/);

const collapse = (text: string) => text.replace(/\s+/g, " ").trim();
const fromMarkdown = (line: string) =>
  collapse(line.replace(/\*\*/g, "").replace(/\*([^*]+)\*/g, "$1").replace(/`/g, ""));

const html = renderToStaticMarkup(createElement(MemoryRouter, null, createElement(TermsPage)));
const pageText = collapse(
  html
    .replace(/<[^>]+>/g, "")
    .replace(/&quot;/g, '"')
    .replace(/&#x27;/g, "'")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&amp;/g, "&"),
);

const clauseLines = lines.filter((line) => /^\d+\.\d+\. /.test(line));

describe("página de Termos espelha o documento legal", () => {
  it("mostra a mesma data de atualização", () => {
    const date = /Última Atualização: (.+?)\*\*/.exec(terms)?.[1];
    expect(date).toBeTruthy();
    expect(pageText).toContain(`Última atualização: ${date}`);
  });

  it("traz o preâmbulo e todos os títulos de seção", () => {
    const preamble = lines.find((line) => line.startsWith("Este contrato de Termos de Uso"));
    expect(pageText).toContain(fromMarkdown(preamble ?? "preâmbulo ausente"));
    const headings = lines.filter((line) => /^## \d+\. /.test(line));
    expect(headings.length).toBe(9);
    for (const heading of headings) {
      expect(pageText.toLowerCase()).toContain(heading.slice(3).toLowerCase());
    }
  });

  it("traz cada cláusula com o texto idêntico", () => {
    expect(clauseLines.length).toBeGreaterThan(30);
    for (const line of clauseLines) {
      expect(pageText).toContain(fromMarkdown(line));
    }
  });

  it("não tem cláusula numerada que o documento não tenha", () => {
    const documentNumbers = clauseLines.map((line) => line.split(" ")[0]);
    const pageNumbers = [...html.matchAll(/<p>(\d+\.\d+\.) /g)].map((match) => match[1]);
    expect(pageNumbers).toEqual(documentNumbers);
  });
});
