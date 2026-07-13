# 📚 Estratégia de Aprendizado — Protocolo Mark (v3.1) — Plataforma de Poker

> **Data:** 2026-07-02
> **Versão:** 3.1 (adicionado Modo Construção)
> **Status:** ATIVO

---

## 🎯 Filosofia Central — Aprender Construindo a Plataforma

**"Mesclar a construção do projeto e o aprendizado."**

O aluno (Leofran) aprende programação **enquanto** constrói o projeto Poker real.
Não há separação entre "aula" e "projeto" — o código do projeto É o material didático.

---

## 🧭 Dois Modos de Operação — Estudo vs. Construção

### 📚 MODO ESTUDO — Tópicos Novos e Complexos
```
Teoria → Exemplo Avançado → Código → Dissecação → Exercício
```
Usado para introduzir conceitos NOVOS. Ritmo mais lento, profundidade máxima.

### ⚡ MODO CONSTRUÇÃO — Avançar a Plataforma de Poker
```
Código → Explicação do Bloco → Analogia → Próximo Bloco
```
Usado para implementar tarefas do backlog. Ritmo rápido, foco em produzir.
**Sem dissecação linha por linha** — o aluno estuda os detalhes por fora.

---

## 📐 Comparativo dos Modos — Estudo vs. Construção

| Aspecto          | Modo Estudo                                  | Modo Construção                                  |
|------------------|----------------------------------------------|--------------------------------------------------|
| **Quando usar**  | Tópico NOVO (ex: primeira vez com JWT)       | Tarefa do backlog (ex: Task #5 Rate Limiting)    |
| **Dissecação**   | ✅ Linha por linha, cada símbolo             | ❌ Não — aluno estuda por fora                    |
| **Analogia**     | ✅ Na teoria                                 | ✅ 1 por bloco de código                          |
| **Exercício**    | ✅ Completo abaixo da dissecação            | ❌ Não — a tarefa É o exercício                   |
| **Ritmo**        | Lento, profundo                              | Rápido, produtivo                                 |
| **Objetivo**     | Entender o conceito                          | Avançar a plataforma                              |

---

## 🔤 Regras de Comunicação — Como Explicar o Código

| Regra           | Descrição                                                                                              |
|-----------------|--------------------------------------------------------------------------------------------------------|
| **Idioma**      | Português brasileiro para explicações. Inglês para código.                                             |
| **Símbolos**    | Todo símbolo (`=>`, `[]`, `{}`, `?:`, etc.) deve ser explicado com analogia do mundo real.              |
| **Código**      | Sempre colocar o bloco de código **abaixo** da explicação, nunca acima.                                |
| **Bloco**       | Explicar intenção do bloco inteiro, não linha por linha.                                               |
| **Dissecação**  | Na dissecação, linha por linha é permitido — é o momento de detalhar CADA caractere.                   |

---

## 🧩 Regra do Exercício de Fixação — Prática no Projeto (NOVA — 2026-07-02)

**Logo abaixo da dissecação detalhada, incluir um exercício completo.**

Estrutura do exercício:

1. **Enunciado** — Variação do exemplo principal (ex: "Modifique a função para que...")
2. **Dica de abordagem** — Por onde começar, qual arquivo abrir primeiro
3. **Onde aplicar** — Arquivo(s) real(is) do projeto para modificar
4. **Resultado esperado** — O que deve acontecer quando estiver correto
5. **Checkpoint de validação** — Comando exato para rodar e confirmar que funcionou

**Propósito:** Fechar o ciclo entender → praticar → fixar.

---

## 📋 Regras de Gestão de Espaço — Organização do Conteúdo

1. **1 exemplo avançado por tópico** (não 6 exercícios graduados)
2. **Código do exemplo vem logo abaixo da explicação**
3. **Dissecação detalhada de cada parte do código**
4. **Foco em mesclar construção do projeto + aprendizado**
5. **Exercício de fixação abaixo da dissecação**

---

## 🗂️ Arquivos da Pasta 05-Documentacao — Mapa de Artefatos

| Arquivo                    | Função                                                                  |
|----------------------------|-------------------------------------------------------------------------|
| `DASHBOARD.md`             | Painel de controle de tarefas (concluídas / backlog)                    |
| `STATUS.md`                | Status SDD (specs implementadas, pendentes)                            |
| `PARAMETROS_ESTUDO.md`     | Protocolo Mark detalhado (regras de entrega)                            |
| `ESTRATEGIA_APRENDIZADO.md` | **ESTE ARQUIVO** — estratégia resumida (fonte da verdade)             |
| `BUSINESS_RULES.md`        | Regras de negócio do poker                                             |
| `ARQUITETURA_MOTOR.md`     | Arquitetura do motor de jogo                                           |
| `STACK.md`                 | Stack tecnológica atual e alvo                                         |

---

## 🔄 Histórico de Versões — Evolução do Protocolo

| Versão | Data       | Mudança                                                                                  |
|--------|------------|------------------------------------------------------------------------------------------|
| 1.0    | 2026-06    | Formato original: 6 etapas (Teoria → Parábolas → 6 exercícios graduados)               |
| 2.0    | 2026-07-02 | Rejeitado pelo aluno. Substituído por 1 exemplo avançado.                                |
| 3.0    | 2026-07-02 | Adicionada regra do Exercício de Fixação abaixo da dissecação.                           |