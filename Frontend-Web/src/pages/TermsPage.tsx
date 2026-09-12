import { Link } from "react-router-dom";

export function TermsPage() {
  return (
    <div className="mx-auto w-full max-w-4xl space-y-6 text-felt-100">
      <div className="border-b border-felt-800 pb-4">
        <Link to="/" className="text-xs text-gold-soft hover:underline">
          ← Voltar à página inicial
        </Link>
        <h1 className="mt-2 text-2xl font-bold text-gold-bright">
          Termos de Uso, Política de Privacidade (LGPD) e Regulamento da Plataforma
        </h1>
        <p className="text-xs text-felt-400">
          Versão Oficial 4.3 • Última atualização: 11 de setembro de 2026 • Zero Tilt Poker & Zero Tilt Academy
        </p>
      </div>

      <div className="zt-panel space-y-6 p-6 leading-relaxed text-sm">
        {/* Sumário Executivo */}
        <div className="rounded-lg border border-gold-soft/30 bg-felt-950/70 p-4 text-xs text-felt-200 space-y-2">
          <p className="font-bold text-gold-bright uppercase tracking-wider">
            📌 Em Linguagem Clara e Direta (Resumo em 1 Minuto):
          </p>
          <ul className="list-disc pl-4 space-y-1">
            <li><strong>Poker é Esporte da Mente:</strong> Habilidade, matemática e controle emocional preponderam sobre a aleatoriedade no longo prazo — sem promessa de ganho.</li>
            <li><strong>Maioridade:</strong> Apenas maiores de 18 anos podem se cadastrar.</li>
            <li><strong>Zero Tilt Academy & Tutor IA:</strong> Os cursos e o simulador com robô são ferramentas de treino teórico e pedagógico. Não há promessa de ganho fácil ou retorno financeiro.</li>
            <li><strong>Rede de Afiliados (2 Níveis):</strong> Você não paga nada para entrar e ninguém ganha comissão por recrutar pessoas. Toda comissão vem exclusivamente do consumo real (rake das mãos jogadas), limitada a exatamente 2 níveis.</li>
            <li><strong>Sua Privacidade (LGPD):</strong> Seus dados pessoais (CPF, e-mail, telefone) nunca são exibidos para quem te indicou e nunca são compartilhados com IAs de terceiros.</li>
            <li><strong>Tolerância Zero com Trapaça:</strong> Conluio, robôs nas mesas públicas ou contas múltiplas geram banimento imediato.</li>
          </ul>
        </div>

        {/* Cláusula 1 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            1. Natureza da Plataforma e Reconhecimento Legal do Poker
          </h2>
          <p>
            1.1. O <strong>Zero Tilt Poker</strong> é uma plataforma digital voltada ao entretenimento, competição esportiva e educação em poker online.
          </p>
          <p>
            1.2. A plataforma trata o poker como <strong>esporte da mente</strong>, em que estratégia, matemática, controle emocional e técnica prevalecem sobre a aleatoriedade no longo prazo, sem promessa de ganho. O jogador deve observar a legislação aplicável à sua jurisdição.
          </p>
          <p>
            1.3. O cadastro é estritamente proibido para menores de 18 (dezoito) anos. Ao criar a conta, o usuário declara, sob as penas da lei, sua maioridade civil e capacidade jurídica plena.
          </p>
        </section>

        {/* Cláusula 2 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            2. Modalidades de Fichas (Play Money vs. Jogo Real)
          </h2>
          <p>
            2.1. <strong>Play Money (Fichas Recreativas):</strong> Fichas virtuais concedidas gratuitamente pela plataforma com renovação diária para fins exclusivamente recreativos e de aprendizado. Não possuem valor de resgate em dinheiro, não são transferíveis entre contas e não geram direito a saques.
          </p>
          <p>
            2.2. <strong>Jogo Real:</strong> Ambiente competitivo com saldo em moeda corrente nacional (BRL). A participação envolve risco de perda de fichas por força da habilidade de jogo dos adversários. A plataforma não garante lucros, rentabilidade ou enriquecimento sob nenhuma hipótese.
          </p>
          <p>
            2.3. Todos os registros de fichas utilizam precisão bancária atômica em inteiros (centavos), garantindo integridade contábil imutável em nosso Ledger.
          </p>
          <p>
            2.4. <strong>Cashback e Loss Deflator (play money):</strong> Após a retirada do rake, o perdedor all-in pode receber de volta parte dos potes líquidos: 7% (equity 56–65,9%), 15% (66–75,9%), 25% (76–85,9%) ou 35% (86%+). Abaixo de 56% não há devolução.
          </p>
        </section>

        {/* Cláusula 3 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            3. Zero Tilt Academy, Tutor Robô de IA e Isenção Educacional
          </h2>
          <p>
            3.1. A <strong>Zero Tilt Academy</strong> e seus simuladores com <strong>Tutor de Inteligência Artificial</strong> são produtos concebidos com fins estritamente pedagógicos e analíticos.
          </p>
          <p>
            3.2. <strong>Isenção de Garantia de Lucros:</strong> A conclusão de aulas, a obtenção de notas superiores a 70% ou o treinamento com os robôs da casa não constituem garantia de vitória ou ganho monetário em mesas competitivas contra outros humanos. O desempenho depende exclusivamente da tomada de decisão pessoal do jogador.
          </p>
          <p>
            3.3. <strong>Proibição de Assistência em Tempo Real (RTA) e Bots Externos:</strong> O uso do Tutor de IA é restrito à área de treino e cursos. É terminantemente proibido utilizar softwares de auxílio em tempo real (RTA), bots externos ou inteligências artificiais acopladas durante partidas regulares ao vivo com outros usuários humanos. A infração enseja banimento sumário.
          </p>
        </section>

        {/* Cláusula 4 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            4. Programa de Afiliados em 2 Níveis (Cláusula Anti-Pirâmide)
          </h2>
          <p>
            4.1. O programa de expansão comunitária da plataforma opera como um <strong>contrato de agenciamento comercial e comissão de vendas em estritos 2 (dois) níveis</strong>.
          </p>
          <p>
            4.2. <strong>Não Caracterização de Pirâmide Financeira:</strong> Em estrito cumprimento à Lei nº 1.521/1951 (crimes contra a economia popular) e às diretrizes da Comissão de Valores Mobiliários (CVM):
          </p>
          <ul className="list-disc pl-5 space-y-1 text-xs text-felt-300">
            <li>Não há cobrança de taxa de adesão, aquisição de kits ou mensalidades obrigatórias para se tornar afiliado;</li>
            <li>Nenhuma comissão ou remuneração é paga pela simples indicação, cadastro ou recrutamento de novos usuários;</li>
            <li>Toda e qualquer bonificação decorre <strong>exclusivamente da prestação e consumo real de serviços</strong> (comissão sobre o rake de mãos jogadas nas mesas);</li>
            <li>A árvore de comissionamento encerra-se peremptoriamente no 2º nível (18% para o 1º nível direto e 12% para o 2º nível indireto, restando 70% para a plataforma). Não existem níveis infinitos, matrizes forçadas ou sistemas binários.</li>
          </ul>
          <p>
            4.3. <strong>Requisito de Ativação e Qualificação Semanal:</strong> Para ter direito e fazer jus ao recebimento de qualquer bonificação de sua rede de afiliados, o usuário deve ter <strong>disputado e concluído no mínimo 100 (cem) mãos</strong> no ciclo semanal vigente, apurado impreterivelmente de <strong>segunda-feira (00:00:00 BRT) até domingo (23:59:59 BRT)</strong>.
          </p>
          <p className="text-xs text-felt-300">
            Caso o afiliado não atinja a meta de 100 mãos no ciclo semanal, a pontuação da rede gerada nessa semana não é creditada, sendo retida pela casa. Não há acúmulo passivo sem trabalho e engajamento efetivo no jogo.
          </p>
          <p>
            4.4. <strong>Data e Periodicidade de Pagamento das Bonificações:</strong> As bonificações decorrentes das semanas em que o afiliado esteve devidamente qualificado são consolidadas ao final do ciclo mensal e <strong>pagas todo dia 25 (vinte e cinco) de cada mês</strong> (ou no primeiro dia útil subsequente).
          </p>
        </section>

        {/* Cláusula 5 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            5. Privacidade e Proteção de Dados Pessoais (LGPD — Lei nº 13.709/2018)
          </h2>
          <p>
            5.1. A plataforma trata dados pessoais em conformidade absoluta com os princípios da finalidade, necessidade e transparência previstos na LGPD.
          </p>
          <p>
            5.2. <strong>Blindagem e Pseudonimização na Rede:</strong> Em nenhuma hipótese o patrocinador de um usuário terá acesso a nome completo, e-mail, telefone, CPF ou dados bancários de seus indicados. O painel da rede exibe estritamente o identificador de jogador mascarado (username) e o volume numérico de comissão agregada.
          </p>
          <p>
            5.3. <strong>Tratamento de Dados no Tutor de IA:</strong> As análises de erros e padrões de jogo realizadas pelo Tutor Robô processam tão somente dados matemáticos da partida (cartas, fichas e apostas). Nenhum dado pessoal identificável é compartilhado ou vendido a provedores externos de inteligência artificial ou corretores de dados.
          </p>
          <p>
            5.4. <strong>Retenção para Fins de Integridade e Legislação Antifraude:</strong> O histórico de mãos disputadas (Hand History) e os registros de transações financeiras são arquivados de forma criptografada pelo prazo de 5 (cinco) anos, em atendimento à legislação de prevenção à lavagem de dinheiro e integridade desportiva (Art. 16 da LGPD).
          </p>
        </section>

        {/* Cláusula 6 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            6. Conduta Esportiva, Conluio e Segurança Antifraude
          </h2>
          <p>
            6.1. São terminantemente proibidas as práticas de:
          </p>
          <ul className="list-disc pl-5 space-y-1 text-xs text-felt-300">
            <li><strong>Conluio (Collusion):</strong> Jogadores combinando estratégias ou compartilhando cartas por qualquer meio;</li>
            <li><strong>Chip Dumping:</strong> Transferência intencional de fichas entre jogadores de forma não competitiva;</li>
            <li><strong>Contas Múltiplas:</strong> Criação de mais de uma conta pela mesma pessoa física ou utilização de dados de terceiros.</li>
          </ul>
          <p>
            6.2. A infração sujeita o infrator ao cancelamento irrevogável da conta, apreensão dos fundos obtidos por meios fraudulentos e eventual comunicação às autoridades competentes.
          </p>
        </section>

        {/* Cláusula 7 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            7. Foro e Aceite Eletrônico
          </h2>
          <p>
            7.1. O aceite eletrônico manifestado no momento do cadastro possui validade jurídica plena e vinculante, nos termos da legislação civil e da Medida Provisória nº 2.200-2/2001.
          </p>
          <p>
            7.2. Fica eleito o Foro da Comarca de São Paulo/SP para dirimir quaisquer controvérsias oriundas deste Contrato, com renúncia expressa a qualquer outro, por mais privilegiado que seja.
          </p>
        </section>

        {/* Cláusula 8 */}
        <section className="space-y-2">
          <h2 className="text-base font-bold text-gold-bright border-b border-felt-800 pb-1">
            8. Torneios, Desconexões e Alterações
          </h2>
          <p>
            8.1. Torneios seguem blinds progressivos e regras de rebuy publicadas; a plataforma pode rebalancear mesas para igualdade de condições.
          </p>
          <p>
            8.2. Desconexão individual durante a mão: a jogada segue o temporizador da mesa (check/fold); a plataforma não se responsabiliza pela conexão do jogador.
          </p>
          <p>
            8.3. A plataforma pode alterar este Contrato mediante publicação da versão atualizada com aviso prévio; o uso continuado implica aceite.
          </p>
        </section>

        <div className="pt-4 border-t border-felt-800 text-center">
          <Link to="/register" className="zt-btn-primary !py-2 !px-6 !text-xs inline-block">
            Entendido, ir para o Cadastro →
          </Link>
        </div>
      </div>
    </div>
  );
}
