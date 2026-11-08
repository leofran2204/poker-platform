// Regras do saque Pix no Jogo Real, espelhando a API (payments_routes.rs):
// o saque só vai para uma chave do MESMO CPF do KYC do jogador. Para chaves
// que não são CPF (aleatória, e-mail, telefone) o CPF do titular é enviado em
// `tax_number`, porque a API e a DePix exigem saber quem recebe.

export type PixKeyType = "cpf" | "evp" | "email" | "phone";

export const PIX_KEY_TYPES: { value: PixKeyType; label: string }[] = [
  { value: "cpf", label: "CPF" },
  { value: "evp", label: "Aleatória (EVP)" },
  { value: "email", label: "E-mail" },
  { value: "phone", label: "Telefone" },
];

export function digitsOnly(value: string): string {
  return value.replace(/\D/g, "");
}

/** CPF com 11 dígitos e dígitos verificadores corretos (mesma regra do KYC). */
export function isValidCpf(value: string): boolean {
  const d = digitsOnly(value);
  if (d.length !== 11 || /^(\d)\1{10}$/.test(d)) return false;
  const nums = [...d].map(Number);
  const check = (len: number) => {
    const sum = nums.slice(0, len).reduce((acc, n, i) => acc + n * (len + 1 - i), 0);
    const rest = sum % 11;
    return rest < 2 ? 0 : 11 - rest;
  };
  return check(9) === nums[9] && check(10) === nums[10];
}

export interface WithdrawInput {
  amountCents: number;
  keyType: PixKeyType;
  key: string;
  /** CPF do titular; ignorado quando a própria chave é CPF. */
  holderCpf: string;
}

export interface WithdrawPayload {
  amount: number;
  pix_key: string;
  pix_key_type: PixKeyType;
  tax_number: string;
}

export type WithdrawBuild = { ok: true; payload: WithdrawPayload } | { ok: false; error: string };

export function buildWithdrawPayload(input: WithdrawInput): WithdrawBuild {
  if (!Number.isInteger(input.amountCents) || input.amountCents <= 0) {
    return { ok: false, error: "Informe um valor de saque maior que zero." };
  }
  const key = input.key.trim();
  if (!key) return { ok: false, error: "Informe a sua chave Pix." };
  if (input.keyType === "cpf") {
    if (!isValidCpf(key)) return { ok: false, error: "CPF da chave Pix inválido." };
    const cpf = digitsOnly(key);
    return {
      ok: true,
      payload: { amount: input.amountCents, pix_key: cpf, pix_key_type: "cpf", tax_number: cpf },
    };
  }
  if (!isValidCpf(input.holderCpf)) {
    return { ok: false, error: "Informe o CPF do titular da chave (o mesmo do seu cadastro)." };
  }
  return {
    ok: true,
    payload: {
      amount: input.amountCents,
      pix_key: key,
      pix_key_type: input.keyType,
      tax_number: digitsOnly(input.holderCpf),
    },
  };
}

/** Mensagens da API ainda em inglês → português claro para o jogador. */
export function translateWithdrawError(message: string): string {
  const m = message.toLowerCase();
  if (m.includes("insufficient wallet balance")) return "Saldo insuficiente para este saque.";
  if (m.includes("tax_number") || m.includes("cpf/cnpj")) {
    return "Informe o CPF do titular da chave (o mesmo do seu cadastro).";
  }
  if (m.includes("must hold 11 digits")) return "CPF da chave Pix inválido.";
  if (m.includes("invalid pix key type")) return "Tipo de chave Pix inválido.";
  if (m.includes("pix key is required")) return "Informe a sua chave Pix.";
  if (m.includes("payouts are unavailable")) {
    return "Saques automáticos indisponíveis no momento. Tente novamente mais tarde ou fale com o suporte.";
  }
  return message;
}
