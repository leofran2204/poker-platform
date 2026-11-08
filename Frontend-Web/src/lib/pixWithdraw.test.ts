import { describe, expect, it } from "vitest";
import { buildWithdrawPayload, isValidCpf, translateWithdrawError } from "./pixWithdraw";

// CPFs de exemplo com dígitos verificadores válidos (gerados, não pertencem a ninguém).
const CPF = "529.982.247-25";
const CPF_DIGITS = "52998224725";

describe("isValidCpf", () => {
  it("aceita CPF com ou sem pontuação", () => {
    expect(isValidCpf(CPF)).toBe(true);
    expect(isValidCpf(CPF_DIGITS)).toBe(true);
  });
  it("recusa dígito verificador errado, repetidos e tamanho errado", () => {
    expect(isValidCpf("52998224724")).toBe(false);
    expect(isValidCpf("11111111111")).toBe(false);
    expect(isValidCpf("1234567890")).toBe(false);
    expect(isValidCpf("12345678000199")).toBe(false); // CNPJ
  });
});

describe("buildWithdrawPayload", () => {
  it("chave CPF: envia só dígitos e usa o próprio CPF como titular", () => {
    const r = buildWithdrawPayload({ amountCents: 5000, keyType: "cpf", key: CPF, holderCpf: "" });
    expect(r).toEqual({
      ok: true,
      payload: { amount: 5000, pix_key: CPF_DIGITS, pix_key_type: "cpf", tax_number: CPF_DIGITS },
    });
  });
  it("chave aleatória/e-mail/telefone: exige e envia o CPF do titular", () => {
    for (const keyType of ["evp", "email", "phone"] as const) {
      const sem = buildWithdrawPayload({ amountCents: 5000, keyType, key: "chave-qualquer", holderCpf: "" });
      expect(sem.ok).toBe(false);
      const com = buildWithdrawPayload({ amountCents: 5000, keyType, key: " chave-qualquer ", holderCpf: CPF });
      expect(com).toEqual({
        ok: true,
        payload: { amount: 5000, pix_key: "chave-qualquer", pix_key_type: keyType, tax_number: CPF_DIGITS },
      });
    }
  });
  it("recusa valor zero, chave vazia e CPF inválido", () => {
    expect(buildWithdrawPayload({ amountCents: 0, keyType: "cpf", key: CPF, holderCpf: "" }).ok).toBe(false);
    expect(buildWithdrawPayload({ amountCents: 100, keyType: "evp", key: "  ", holderCpf: CPF }).ok).toBe(false);
    expect(buildWithdrawPayload({ amountCents: 100, keyType: "cpf", key: "123", holderCpf: "" }).ok).toBe(false);
  });
});

describe("translateWithdrawError", () => {
  it("traduz mensagens em inglês da API", () => {
    expect(translateWithdrawError("Insufficient wallet balance for PIX withdrawal")).toBe(
      "Saldo insuficiente para este saque.",
    );
    expect(translateWithdrawError("tax_number (CPF/CNPJ) is required for non-CPF PIX keys")).toContain(
      "CPF do titular",
    );
  });
  it("mantém mensagens já em português (ex.: regra de mesma titularidade)", () => {
    const msg = "O saque só pode ir para uma chave PIX do mesmo CPF do seu cadastro (KYC).";
    expect(translateWithdrawError(msg)).toBe(msg);
  });
});
