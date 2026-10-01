# Plano de mercado — Agente ZT Poker

**Decisão comercial:** 2026-09-27

**Público:** fundador e agentes aprovados
**Estágio atual:** Play Money em demo; qualquer comissão em dinheiro depende da operação real autorizada e dos controles aplicáveis.

---

## 1. Regra em uma frase

O **Agente ZT Poker** recebe **30% da receita líquida atribuível aos jogadores que indicou diretamente** e chega a **35% no mês em que cumprir a meta de desempenho**.

Não existe segundo nível. Não existe comissão por cadastro. Não existe taxa para ser agente.

---

## 2. Percentuais

| Situação no mês | Agente | Casa |
|---|---:|---:|
| Sem agente | 0% | 100% |
| Agente aprovado | **30% do NGR direto** | **70% do NGR direto** |
| Agente atingiu a meta mensal | **35% do NGR direto** | **65% do NGR direto** |

Os 5 pontos percentuais adicionais incidem sobre todo o NGR direto elegível do mês. A meta é definida e registrada antes do início do ciclo e não pode ser alterada retroativamente.

---

## 3. Base de cálculo

```text
NGR direto =
  rake de cash dos jogadores diretos
+ fees de torneios dos jogadores diretos
- recompensas e benefícios concedidos a esses jogadores
- estornos e chargebacks atribuíveis
- tributos e custos de pagamento diretamente atribuíveis
```

A comissão nunca incide sobre depósito, saldo, buy-in, prêmio, cadastro ou perda do jogador. O contrato e o painel devem mostrar as deduções usadas no fechamento.

Exemplo: NGR direto de R$ 1.000 no mês.

| Resultado | Comissão | Casa |
|---|---:|---:|
| Sem atingir a meta | R$ 300 | R$ 700 |
| Atingindo a meta | R$ 350 | R$ 650 |

---

## 4. Quem pode ser Agente ZT

O agente é aprovado pela plataforma e assume quatro tarefas:

1. apresentar a ZT Poker;
2. ajudar o jogador no primeiro acesso;
3. organizar mesas e horários;
4. prestar suporte inicial e encaminhar problemas.

O agente recebe apenas pelos jogadores ligados diretamente ao seu código. O vínculo usa `users.sponsored_by`, é único e não pode ser trocado para desviar comissão.

O agente não pode receber depósitos, pagar saques, conceder crédito, movimentar carteira de jogador, pedir senha ou código de autenticação.

---

## 5. Meta de desempenho

A meta mensal usa **NGR direto elegível**, porque mede resultado real e preserva a margem da plataforma.

- A plataforma publica o valor da meta antes do mês começar.
- O painel mostra o progresso do agente.
- Abaixo da meta: 30%.
- Na meta ou acima: 35%.
- Fraude, autoindicação ou violação comercial retira a parcela irregular da apuração e segue o processo de auditoria; não há punição financeira arbitrária.

O valor inicial da meta só deve ser fixado depois de medir custos, recompensas e margem da operação. O percentual aprovado já está definido; o valor da meta é um parâmetro operacional.

---

## 6. Fechamento

- Ciclo: mês civil em `America/Sao_Paulo`.
- Fechamento: manual, disponível a partir do dia 25 do mês seguinte, após conciliação do ciclo anterior.
- Base: ledger auditável por jogador e origem (`rake` ou `fee`).
- Ajustes: estornos entram identificados, sem apagar lançamentos anteriores.
- Deduções acima da receita: comissão zero e déficit transportado ao próximo ciclo, mantendo Play e Real separados.
- Metas sem valor positivo: percentual base de 30%. O administrador define as próximas metas antes do início do mês.
- Mesas B2B: a base dos agentes usa somente a parcela de receita retida pela plataforma, após o repasse ao clube.
- Conferência: o administrador registra os custos atribuíveis, consulta a prévia e confirma a conciliação; valores alterados exigem nova confirmação. O fechamento fica disponível a partir do dia 25 do mês seguinte, em ordem cronológica.
- Play Money: apuração somente em ZT Points, tickets, seats ou ranking; sem conversão em PIX.
- Jogo Real: pagamento somente quando a operação correspondente estiver autorizada e conciliada.

---

## 7. Painel do agente

O painel deve mostrar somente:

- código e link do agente;
- jogadores diretos;
- NGR direto do mês;
- deduções do período;
- percentual atual: 30% ou 35%;
- progresso da meta;
- comissão prevista e comissão fechada.

Não há árvore, segundo nível, VL2 ou comissão sobre indicações feitas pelos jogadores do agente.

---

## 8. Comunicação

Mensagem curta:

```text
Agente ZT Poker

Você cuida dos jogadores que trouxe diretamente.
Recebe 30% da receita líquida gerada por eles.
Se bater a meta mensal, recebe 35% naquele mês.
Sem segundo nível e sem taxa de adesão.
```

Evitar promessas de renda, ganho garantido, comissão por cadastro e qualquer orientação para pagamentos por fora da plataforma.

---

## 9. Transição técnica

O modelo foi implementado localmente na migration `060` e no código da API e do frontend:

1. novas linhas usam `program_version = 2` e somente o patrocinador direto;
2. o ledger 18%/12% anterior permanece preservado como histórico `program_version = 1`;
3. cash e fee de torneio acumulam receita bruta por mês e por carteira;
4. deduções identificadas formam o NGR conciliado;
5. o fechamento idempotente aplica 30% ou 35% e credita uma única vez;
6. Play Money fecha em ZT Points e Jogo Real em saldo de comissão separado;
7. o painel do agente mostra carteira direta, NGR, meta, projeção e histórico;
8. o painel administrativo aprova/suspende agentes, fixa a meta, registra deduções e fecha o mês.

A demo pública continua com o comportamento do último deploy até a migration `060` e o novo build serem publicados. Linhas históricas não são recalculadas nem somadas ao novo programa.

---

## 10. Critério de sucesso

O programa funciona quando aumenta jogadores ativos e NGR sem consumir mais de 35% do NGR direto em comissão.

Indicadores mensais:

- jogadores diretos ativos;
- NGR direto;
- retenção em 30 dias;
- comissão como percentual do NGR;
- custo de recompensas;
- estornos e chargebacks;
- chamados e incidentes por agente.

---

## 11. Checklist do dono

- [x] Nome: **Agente ZT Poker**
- [x] Um único nível
- [x] Base: NGR dos jogadores diretos
- [x] Comissão base: 30%
- [x] Comissão por desempenho: 35%
- [ ] Fixar a primeira meta mensal com dados de margem
- [x] Implementar o novo ledger e o fechamento mensal
- [x] Atualizar o painel e as mensagens públicas
- [ ] Publicar a migration `060` e registrar a data efetiva do corte em produção

---

*Regra comercial aprovada: 30% direto + 5 pontos percentuais por desempenho. Simples, mensal e limitado a um nível.*

<!-- DOCUMENTATION_SYNC:START -->
> **S26** (2026-10-01) — demo `zerotiltpoker.net` · sem certificação de produção · PIX automático ligado (DePix reconciliado).
> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md).
<!-- DOCUMENTATION_SYNC:END -->
