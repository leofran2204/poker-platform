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
> **Estado operacional sincronizado (2026-09-04):** S21 — Texas Hold’em rename + FT Short Deck 8-max + Omaha 5-max + Pineapple 6-max + Short Deck ranking trips>straight + torneio agendado 21:30 SP auto-start 5 + Pix Leofran + saque 24h + lobby max sempre + sim 100k/mesa + PM 150+150 sem rebuy (ilimitado com saldo, play money) **Sem certificação de produção; o código rejeita PIX em modo production. Deploy público: VPS Hostinger (demo/staging) com domínio zerotiltpoker.net. Staging/demo apenas; não alegar Launch Ready de produção.** Stack Docker local 4/4 healthy e VPS Hostinger 4/4 healthy. Migrations 001–044 na VPS (cash Texas SD 8-max + Omaha 5-max + Pineapple 6-max + Texas rename + FT 8 + scheduled 21:30 + PM 150+150 + restore catálogo + Dockerfile cache). PM duas carteiras R$150 sem rebuy (ilimitado com saldo). Motor short_deck_massive + tournament_to_champion PASS (Texas/Omaha 5/Pineapple 6 até 1 campeão; flush>FH e trips>straight). VPS 2h real 100 contas: 980 mãos R$135,11 rake, 4 campeões MTT. Simulado Motor-Rust/src/bin/simulated_100.rs 100k/mesa (400k total). Lobby GET /api/lobby/tables lista mesas OPEN mesmo lotadas com X-max sempre. MTT site: inscrição + horário agendado + popup FT; gameplay_ready=false (sem WS de torneio). Health público OK. Recebedor manual: Leofran, chave 6eefcd53-686e-42d4-a062-03751336251c (PLAY_MONEY_PIX_KEY). Saque: informar chave Pix própria, recebimento em até 24h. A VPS permanece no padrão seguro PIX mock. DePix existe somente em Sandbox não produtivo, com chave sk_test_, allowlist de depositante, idempotência, HMAC com janela temporal, deduplicação de eventos e crédito apenas em checkout.completed. O CPF/CNPJ é encaminhado ao provedor sem persistência local. Depósito manual continua como fallback; não há saque automático. Mesas com dono único por processo; settlement assinado (HMAC) na liquidação.
>
> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.
<!-- DOCUMENTATION_SYNC:END -->
