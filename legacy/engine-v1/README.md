# legacy/engine-v1

Arquivo histórico do **motor original em centavos flutuantes (`f64`)**, anterior à migração
para `u64` centavos inteiros (S06) e ao motor canônico atual em `Motor-Rust/`.

- **Não compila no build atual e não entra em CI/validação** — movido para cá em 2026-09-06
  para que cada assunto more no seu lugar (reorganização do monorepo).
- A única ferramenta viva da raiz é `src/bin/documentation_sync.rs`
  (crate `poker-tooling`, `cargo run --bin documentation-sync -- --check`).
- Todo o histórico permanece no git (`git log -- legacy/engine-v1`).
- Referências antigas a `use poker_engine::...` dentro destes arquivos apontam para
  este engine arquivado, **não** para o `poker_engine` (lib) do `Motor-Rust/`.

Não ressuscitar sem decisão explícita de arquitetura.
