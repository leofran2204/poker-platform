import type { ReactNode } from "react";
import { Link } from "react-router";

// Texto espelhado de Documentacao/TERMOS_DE_USO_E_SERVICO.md (dono do texto legal).
// TermsPage.test.ts confere data e cláusulas contra o documento: altere os dois juntos.

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="space-y-2">
      <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">{title}</h2>
      {children}
    </section>
  );
}

export function TermsPage() {
  return (
    <div className="mx-auto w-full max-w-4xl space-y-6 text-felt-100">
      <div className="border-b border-felt-800 pb-4">
        <Link to="/" className="text-xs text-gold-soft hover:underline">
          ← Voltar à página inicial
        </Link>
        <h1 className="mt-2 text-2xl font-bold text-gold-bright">
          Termos de Uso e Condições do Serviço
        </h1>
        <p className="text-xs text-felt-400">
          Última atualização: 8 de novembro de 2026 • Zero Tilt Poker & Zero Tilt Academy
        </p>
      </div>

      <div className="zt-panel space-y-6 p-6 leading-relaxed text-sm">
        {/* Sumário — não substitui as cláusulas abaixo */}
        <div className="rounded-lg border border-gold-soft/30 bg-felt-950/70 p-4 text-xs text-felt-200 space-y-2">
          <p className="font-bold text-gold-bright uppercase tracking-wider">
            📌 Em linguagem clara (resumo em 1 minuto):
          </p>
          <ul className="list-disc pl-4 space-y-1">
            <li><strong>Esporte da mente:</strong> habilidade, matemática e controle emocional pesam mais que a sorte no longo prazo — sem promessa de ganho.</li>
            <li><strong>Maioridade:</strong> apenas maiores de 18 anos, uma conta por pessoa.</li>
            <li><strong>Jogo Real:</strong> depósito, entrada e saque exigem verificação de identidade aprovada; o saque só vai para chave PIX do mesmo CPF verificado.</li>
            <li><strong>Mesma rede, mesas diferentes:</strong> nas mesas cash, duas contas do mesmo IP ou da mesma sub-rede (por exemplo, o mesmo Wi-Fi) não sentam na mesma mesa.</li>
            <li><strong>Loss Deflator:</strong> vale em todas as modalidades, no cash e nos torneios, em Play Money e Jogo Real, e sai do próprio pote.</li>
            <li><strong>Baralho:</strong> embaralhado por gerador criptográfico e mão assinada; a verificação pública pelo jogador (Provably Fair) ainda não está disponível.</li>
            <li><strong>Agente ZT Poker:</strong> um nível direto, sem taxa de adesão e sem comissão por cadastro.</li>
            <li><strong>Tolerância zero com trapaça:</strong> robôs, assistência em tempo real, conluio ou contas múltiplas levam à suspensão.</li>
          </ul>
        </div>

        <p>
          Este contrato de Termos de Uso ("Contrato") rege o uso da plataforma de poker online ("Plataforma") por qualquer usuário cadastrado ("Jogador"). Ao criar uma conta ou utilizar os serviços da Plataforma, o Jogador aceita integralmente os termos aqui descritos.
        </p>

        <Section title="1. Elegibilidade, Maioridade e Reconhecimento Legal do Poker">
          <p>1.1. O acesso à Plataforma é restrito a indivíduos maiores de 18 (dezoito) anos de idade com plena capacidade civil. Ao cadastrar-se, o Jogador declara sob as penas da lei a veracidade de suas informações.</p>
          <p>1.2. A Plataforma trata o poker como esporte da mente, em que habilidade, matemática e controle emocional preponderam sobre a aleatoriedade no longo prazo, sem qualquer promessa de ganho. O Jogador deve observar a legislação aplicável à sua jurisdição.</p>
          <p>1.3. É estritamente proibida a criação de múltiplas contas por um mesmo indivíduo (<em>multi-accounting</em>).</p>
          <p>1.4. O uso de Play Money pode permanecer disponível durante a análise da conta. Depósitos, novas entradas e ações com saldo de Jogo Real dependem de verificação de identidade aprovada e dos controles de proteção aplicáveis.</p>
        </Section>

        <Section title="2. Regras de Conduta, Antifraude e Proibição de RTA">
          <p>2.1. <strong>Proibição de Assistência em Tempo Real (RTA) e Bots:</strong> É expressamente vedado o uso de robôs (<em>bots</em>), inteligência artificial em tempo real (RTA), softwares de automação ou assistência ilícita durante o jogo ao vivo em mesas com outros jogadores humanos. Ferramentas de estudo restringem-se à área de treino da Zero Tilt Academy (quiz e exercícios), nunca durante mãos ao vivo.</p>
          <p>2.2. <strong>Proibição de Conluio (<em>Collusion</em>):</strong> É proibida qualquer combinação prévia, compartilhamento de informações de cartas ou cooperação ilícita entre Jogadores na mesma mesa para obter vantagem indevida sobre terceiros.</p>
          <p>2.3. <strong>Restrição de IP e Sub-Rede:</strong> Nas mesas cash, o sistema antifraude da Plataforma recusa automaticamente a entrada de um Jogador na mesa em que outra conta já tenha assento ativo a partir do mesmo endereço IP público ou da mesma sub-rede (/24 em IPv4; /64 em IPv6). Jogadores que compartilham a mesma rede, como o mesmo Wi-Fi, devem escolher mesas diferentes. A Plataforma guarda apenas impressões criptográficas desses endereços, e somente enquanto o assento estiver ativo.</p>
          <p>2.4. <strong>Penalidades:</strong> O descumprimento destas regras resultará na suspensão imediata da conta, perda do saldo de prêmios obtidos ilicitamente e congelamento dos fundos no Ledger imutável para investigação.</p>
        </Section>

        <Section title="3. Zero Tilt Academy, Ferramentas de Estudo e Isenção de Ganhos">
          <p>3.1. A Zero Tilt Academy, suas mesas de estudo com bots didáticos e eventuais ferramentas de coach virtual, quando disponibilizadas, têm fins estritamente pedagógicos e analíticos. O coach limita-se à revisão de mãos já encerradas.</p>
          <p>3.2. A conclusão de módulos ou aprovação em testes de fixação não assegura garantia, promessa de rentabilidade ou ganhos financeiros em mesas reais. O poker é um esporte de habilidade cujo resultado depende de cada decisão individual.</p>
        </Section>

        <Section title="4. Programa Agente ZT Poker">
          <p>4.1. Quando ativado pela Plataforma, o programa opera em <strong>um único nível</strong>. O Agente ZT recebe 30% da receita líquida atribuível aos jogadores ligados diretamente ao seu código e mais 5 (cinco) pontos percentuais no mês em que atingir a meta de desempenho informada antes do início do ciclo.</p>
          <p>4.2. <strong>Cumprimento à Lei nº 1.521/1951:</strong> Não há cobrança de taxa de adesão ou compra de kits obrigatórios. Nenhuma bonificação é gerada pelo simples recrutamento ou cadastro de novos membros.</p>
          <p>4.3. A base de cálculo é o NGR direto: rake e fees atribuíveis, descontados os benefícios concedidos aos jogadores, estornos, chargebacks, tributos e custos de pagamento diretamente atribuíveis. Não há remuneração sobre cadastro, depósito, saldo, buy-in, prêmio ou perda do jogador.</p>
          <p>4.4. Indicações feitas pelos jogadores do agente não geram comissão. Autoindicação, fraude e movimentação artificial são excluídas da apuração e submetidas à auditoria.</p>
          <p>4.5. <strong>Fechamento:</strong> a apuração usa o mês civil em <code>America/Sao_Paulo</code> e pode ser consolidada pelo administrador a partir do dia 25 do mês seguinte, após a conciliação do ciclo anterior. No Play Money, qualquer benefício permanece virtual e sem conversão em dinheiro ou PIX.</p>
        </Section>

        <Section title="5. Privacidade e Proteção de Dados (LGPD — Lei nº 13.709/2018)">
          <p>5.1. A Plataforma processa dados pessoais exclusivamente para execução de contrato, segurança e combate a fraudes.</p>
          <p>5.2. <strong>Garantia de Pseudonimização:</strong> Patrocinadores na rede não têm acesso a dados pessoais sensíveis, CPF, e-mail, telefone ou dados bancários de seus indicados.</p>
          <p>5.3. <strong>Dados nas ferramentas de estudo:</strong> O servidor de treino processa o cenário, a distribuição, as ações e as respostas aos exercícios. As anotações do caderno são salvas neste navegador e podem ser exportadas pelo aluno. A mesa de estudo não envia esses dados a serviços externos de inteligência artificial. Quando disponibilizado, o coach virtual deve analisar somente mãos encerradas do próprio Jogador, sem assistência em tempo real e sem revelar cartas privadas de terceiros. Qualquer uso futuro de provedor externo exige atualização prévia desta política e informação ao titular.</p>
          <p>5.4. Em cumprimento à legislação antifraude, o histórico de partidas (Hand History) é retido por 5 anos (Art. 16 da LGPD).</p>
          <p>5.5. No KYC básico/manual, a Plataforma guarda nome legal, data de nascimento, estado da análise e somente HMAC do CPF com os quatro últimos dígitos; o CPF integral não é persistido nesse fluxo. O acesso administrativo deve ser limitado à análise e segurança da conta.</p>
        </Section>

        <Section title="6. Transações Financeiras e Saldos">
          <p>6.1. Todos os saldos e apostas são registrados de forma atômica no Ledger Financeiro Imutável em centavos de moeda corrente.</p>
          <p>6.2. Os depósitos e saques só serão processados após a verificação de segurança da conta.</p>
          <p>6.3. <strong>Movimentação automática via DePix:</strong> depósitos de até R$ 50 são creditados quando o provedor confirma o PIX (<code>processing</code>); o saque permanece bloqueado até a liquidação final (<code>completed</code>). Acima de R$ 50 o saldo só entra no <code>completed</code>. O crédito final corresponde ao <strong>valor líquido recebido do provedor, deduzidas as taxas dele</strong> (visível no extrato da carteira). Se o PIX for cancelado ou expirar depois do crédito provisório, a Plataforma estorna o que ainda estiver na carteira (sem saldo negativo). Saques exigem verificação de identidade aprovada e vão exclusivamente para chave PIX do mesmo CPF informado nessa verificação; chaves de terceiros e de CNPJ são recusadas sem reservar saldo. Saques automáticos seguem até o teto publicado; acima dele, o saque aguarda revisão manual em até 24h. Falhas do provedor de saque recreditam o saldo reservado, sem estorno PIX.</p>
          <p>6.4. <strong>Cashback e Loss Deflator:</strong> O benefício vale em todas as modalidades (Texas Hold’em, Texas Hold’em Short Deck, Omaha 4 e Brazilian Pineapple), nas mesas cash e nos torneios, tanto em Play Money quanto em Jogo Real. Após a retirada do rake do main pot e de cada side pot (torneios não cobram rake por mão), a Plataforma devolve ao perdedor all-in parte dos potes líquidos em que era elegível: 7% para equity de 56,0% a 65,9%; 15% de 66,0% a 75,9%; 25% de 76,0% a 85,9%; e 35% de 86,0% a 99,9% (abaixo de 56,0% não há devolução; 100% não tem perdedor possível). A equity considerada é a do instante em que o all-in é pago; no Omaha 4 ela respeita a regra de exatamente 2 cartas privadas e 3 comunitárias; no Brazilian Pineapple, usa as cartas privadas que cada jogador tinha naquele instante (3 antes do descarte do river, 2 depois dele), conta no máximo 2 delas na melhor mão e não considera trocas futuras (quem está all-in não troca mais). Quando o número de desfechos possíveis é grande demais para contagem exata, a equity é estimada por simulação reproduzível: a mesma mão sempre produz o mesmo resultado. O valor devolvido sai do próprio pote, descontado de quem o venceu; nos torneios, é pago em fichas do torneio.</p>
          <p>6.5. O Jogador pode definir limites voluntários de depósito, perda e tempo diário. Reduções entram em vigor imediatamente; aumento ou remoção aguarda período de resfriamento de 24 horas.</p>
          <p>6.6. A autoexclusão temporária ou permanente bloqueia imediatamente depósitos, novas entradas e ações de Jogo Real. Não existe cancelamento antecipado pelo fluxo normal da Plataforma.</p>
        </Section>

        <Section title="7. Transparência e Integridade do Baralho">
          <p>7.1. Cada mão é embaralhada com o algoritmo Fisher-Yates alimentado pelo gerador de números aleatórios criptograficamente seguro do sistema operacional do servidor. Nem a Plataforma nem os Jogadores escolhem ou conhecem a ordem das cartas antes de elas serem distribuídas.</p>
          <p>7.2. Ao término de cada mão, o histórico e a liquidação (potes, rake, eventual Loss Deflator e pagamentos) são gravados e assinados com HMAC-SHA256, para auditoria e resolução de disputas.</p>
          <p>7.3. A verificação pública do baralho pelo próprio Jogador (<em>Provably Fair</em>, com publicação prévia do hash da semente e revelação posterior) ainda não está disponível. Quando for disponibilizada, estes Termos serão atualizados.</p>
        </Section>

        <Section title="8. Foro e Validade do Aceite Eletrônico">
          <p>8.1. O aceite digital manifestado no cadastro vincula as partes nos termos da MP nº 2.200-2/2001 e Código Civil, elegendo-se o Foro da Comarca de São Paulo/SP.</p>
        </Section>

        <Section title="9. Torneios, Desconexões e Alterações">
          <p>9.1. Torneios seguem blinds progressivos e regras de rebuy publicadas; a Plataforma pode rebalancear mesas para igualdade de condições.</p>
          <p>9.2. Desconexão individual durante a mão: a jogada segue o temporizador da mesa (check/fold); a Plataforma não se responsabiliza pela conexão do Jogador.</p>
          <p>9.3. A Plataforma pode alterar este Contrato mediante publicação da versão atualizada com aviso prévio; o uso continuado implica aceite.</p>
        </Section>

        <div className="pt-4 border-t border-felt-800 text-center">
          <Link to="/register" className="zt-btn-primary py-2! px-6! text-xs! inline-block">
            Entendido, ir para o Cadastro →
          </Link>
        </div>
      </div>
    </div>
  );
}
