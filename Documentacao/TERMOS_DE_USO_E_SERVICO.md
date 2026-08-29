# TERMOS DE USO E CONDIÇÕES DO SERVIÇO - PLATAFORMA DE POKER ONLINE

**Última Atualização: 24 de Julho de 2026**

Este contrato de Termos de Uso ("Contrato") rege o uso da plataforma de poker online ("Plataforma") por qualquer usuário cadastrado ("Jogador"). Ao criar uma conta ou utilizar os serviços da Plataforma, o Jogador aceita integralmente os termos aqui descritos.

---

## 1. ELEGIBILIDADE E CADASTRO
1.1. O acesso à Plataforma é restrito a indivíduos maiores de 18 (dezoito) anos de idade ou com idade legal equivalente na jurisdição de sua residência.
1.2. O Jogador é responsável por fornecer informações pessoais verdadeiras, exatas e atualizadas durante o cadastro e processo de verificação de identidade (KYC).
1.3. É estritamente proibida a criação de múltiplas contas por um mesmo indivíduo (*multi-accounting*). Múltiplas contas identificadas serão permanentemente encerradas.

---

## 2. REGRAS DE CONDUTA E ANTIFRAUDE
2.1. **Proibição de Assistência em Tempo Real (RTA) e Bots:** É expressamente vedado o uso de robôs (*bots*), inteligência artificial em tempo real (RTA), softwares de automação ou assistência ilícita durante o jogo ao vivo.
2.2. **Proibição de Conluio (*Collusion*):** É proibida qualquer combinação prévia, compartilhamento de informações de cartas ou cooperação ilícita entre Jogadores na mesma mesa para obter vantagem indevida sobre terceiros.
2.3. **Restrição de IP e Sub-Rede:** O sistema antifraude da Plataforma bloqueará automaticamente o assento simultâneo de Jogadores sob o mesmo endereço IP ou sub-rede na mesma mesa.
2.4. **Penalidades:** O descumprimento destas regras resultará na suspensão imediata da conta, perda do saldo de prêmios obtidos ilicitamente e congelamento dos fundos no Ledger imutável para investigação.

---

## 3. TRANSAÇÕES FINANCEIRAS E SALDOS
3.1. Todos os saldos e apostas são registrados de forma atômica no Ledger Financeiro Imutável em centavos de moeda corrente.
3.2. Os depósitos e saques só serão processados após a verificação de segurança da conta.
3.3. **Cashback e Loss Deflator:** No estágio atual, o benefício opera somente com fichas playmoney. Após a retirada do rake do main pot e de cada side pot, a Plataforma pode devolver ao perdedor all-in parte dos potes líquidos em que era elegível: 7% para equity de 56,0% a 65,9%; 15% de 66,0% a 75,9%; 25% de 76,0% a 85,9%; e 35% a partir de 86,0%. Abaixo de 56,0% não há devolução. A equity considerada é a do instante em que o all-in é pago; a fase da mão não determina a faixa.

---

## 4. REGRAS DE TORNEIOS E CASH GAMES
4.1. Torneios (*MTT* e *Sit & Go*) seguem a estrutura oficial de aumento progressivo de blinds e regras de re-buys pré-estabelecidas.
4.2. **Balanceamento de Mesas:** À medida que Jogadores são eliminados em torneios, a Plataforma realocará automaticamente os participantes entre as mesas ativas via algoritmo de balanceamento para manter a igualdade de condições.
4.3. **Desconexões:** A Plataforma não se responsabiliza por falhas ou interrupções na conexão de internet pessoal do Jogador. Em caso de desconexão individual durante a mão, a jogada será tratada como *Check/Fold* conforme o temporizador padrão da mesa.

---

## 5. TRANSPARÊNCIA E PROVABLY FAIR
5.1. A Plataforma utiliza um sistema de embaralhamento criptograficamente justo (*Provably Fair*) baseado no algoritmo ChaCha8 com comprometimento de semente de servidor (*Server Seed Hash*).
5.2. Ao término de cada mão, o Jogador pode solicitar o registro do histórico para reconstruir o embaralhamento e verificar a integridade da distribuição das cartas.

---

## 6. PROPRIEDADE INTELECTUAL
6.1. Todos os direitos de propriedade intelectual da Plataforma, software, gráficos, interfaces e código-fonte pertencem exclusivamente à operadora da Plataforma, protegidos por lei.

---

## 7. ALTERAÇÕES DOS TERMOS
7.1. A Plataforma reserva-se o direito de alterar este Contrato a qualquer momento, mediante publicação da versão atualizada com notificação prévia aos Jogadores.

<!-- DOCUMENTATION_SYNC:START -->
> **Estado operacional sincronizado (2026-08-29):** S18 — Catálogo cash NLHE+Short Deck+SD Omaha (PM×Real); frentes fixas; motor short_deck_omaha; notícias com capa temática; testes 10k/mesa + e2e seeded Real/PM. Demo VPS zerotiltpoker.net. **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** VPS stack healthy (postgres, redis, api, frontend/Caddy). Migrations 001–025. Presence API no ar. Motor: cash_catalog_10k_hands PASS (4 configs × 10k); short_deck_massive PASS; tournament_engine 954 ok. Smoke live seeded Real+PM: join+≥1 mão em cada mesa OPEN; inscrição torneios OK. Mock/auto PIX bloqueado para gaming em vários PSPs. Fluxo vigente: Pedir fichas (depósito manual) + comprovante + aprovação admin. Nenhum gateway de saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
