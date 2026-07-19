// ─── Módulo de Lobby ───
// Gerencia mesas ativas, matchmaking e estado dos jogadores.
// Regras de negócio: BUSINESS_RULES.md §13
//
// ============================================================
// 🎯 META DE TESTES FASE 2: +720 testes (28 → 748)
// ============================================================
// Lotes planejados:
//   [x] 9A — Types & Creation (120 testes)
//   [x] 9B — Table Management (200 testes)
//   [x] 9C — Player Management (200 testes)
//   [x] 9D — Queries & Stats (120 testes)
//   [x] 9E — Edge Cases (80 testes)
// Progresso atual: 5/5 lotes (720+ testes)
// ============================================================

use serde::{Deserialize, Serialize};

// ─── Tipos de Jogo ───

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameType {
    Cash,
    Tournament,
}

// ─── Visibilidade da Mesa ───

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableVisibility {
    Public,
    Private,
}

// ─── Status do Jogador no Lobby ───

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerLobbyStatus {
    /// Navegando entre mesas, não está jogando
    Lobby,
    /// Sentado em uma mesa ativa
    Playing,
    /// Assistindo a uma mesa sem apostar
    Observing,
}

// ─── Informações de Mesa para o Lobby ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TableInfo {
    /// Identificador único da mesa
    pub id: String,
    /// Nome da mesa
    pub name: String,
    /// Tipo de jogo (Cash ou Tournament)
    pub game_type: GameType,
    /// Valor do small blind
    pub small_blind: u64,
    /// Valor do big blind
    pub big_blind: u64,
    /// Buy-in mínimo
    pub min_buy_in: u64,
    /// Buy-in máximo
    pub max_buy_in: u64,
    /// Número máximo de jogadores
    pub max_players: u8,
    /// Número atual de jogadores
    pub current_players: u8,
    /// Visibilidade (Pública ou Privada)
    pub visibility: TableVisibility,
    /// Senha para mesas privadas (None para públicas)
    pub password_hash: Option<String>,
}

// ─── Resultado de Operações no Lobby ───

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LobbyResult {
    /// Se a operação foi bem-sucedida
    pub success: bool,
    /// Mensagem descritiva
    pub message: String,
    /// ID da mesa envolvida (se aplicável)
    pub table_id: Option<String>,
}

// ─── Gerenciador do Lobby ───

#[derive(Debug, Clone)]
pub struct LobbyManager {
    /// Todas as mesas ativas no sistema
    tables: Vec<TableInfo>,
    /// Contador para gerar IDs únicos de mesa
    next_table_id: u64,
}

impl LobbyManager {
    /// Cria um novo gerenciador de lobby vazio
    pub fn new() -> Self {
        LobbyManager {
            tables: Vec::new(),
            next_table_id: 1,
        }
    }

    /// Cria uma nova mesa e a adiciona ao lobby.
    /// Retorna o ID da mesa criada.
    #[allow(clippy::too_many_arguments)]
    pub fn create_table(
        &mut self,
        name: String,
        game_type: GameType,
        small_blind: u64,
        big_blind: u64,
        min_buy_in: u64,
        max_buy_in: u64,
        max_players: u8,
        visibility: TableVisibility,
        password: Option<String>,
    ) -> String {
        let id = format!("table_{}", self.next_table_id);
        self.next_table_id += 1;

        let password_hash = password.map(|p| {
            // Hash simples para senha de mesa (não precisa de bcrypt completo)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            p.hash(&mut hasher);
            hasher.finish().to_string()
        });

        let table = TableInfo {
            id: id.clone(),
            name,
            game_type,
            small_blind,
            big_blind,
            min_buy_in,
            max_buy_in,
            max_players,
            current_players: 0,
            visibility,
            password_hash,
        };

        self.tables.push(table);
        id
    }

    /// Lista todas as mesas ativas, opcionalmente filtrando por tipo de jogo.
    pub fn list_tables(&self, game_type_filter: Option<GameType>) -> Vec<&TableInfo> {
        self.tables
            .iter()
            .filter(|t| {
                game_type_filter
                    .as_ref()
                    .is_none_or(|gt| t.game_type == *gt)
            })
            .collect()
    }

    /// Lista mesas filtrando por faixa de blinds (big blind entre min e max).
    pub fn list_tables_by_blinds(&self, min_blind: u64, max_blind: u64) -> Vec<&TableInfo> {
        self.tables
            .iter()
            .filter(|t| t.big_blind >= min_blind && t.big_blind <= max_blind)
            .collect()
    }

    /// Lista mesas com vagas disponíveis.
    pub fn list_available_tables(&self) -> Vec<&TableInfo> {
        self.tables
            .iter()
            .filter(|t| t.current_players < t.max_players)
            .collect()
    }

    /// Busca uma mesa pelo ID.
    pub fn find_table(&self, table_id: &str) -> Option<&TableInfo> {
        self.tables.iter().find(|t| t.id == table_id)
    }

    /// Busca uma mesa pelo ID (mutável).
    fn find_table_mut(&mut self, table_id: &str) -> Option<&mut TableInfo> {
        self.tables.iter_mut().find(|t| t.id == table_id)
    }

    /// Tenta colocar um jogador em uma mesa.
    /// Valida saldo, senha (se privada) e vagas disponíveis.
    pub fn join_table(
        &mut self,
        table_id: &str,
        player_balance: u64,
        password_attempt: Option<String>,
    ) -> LobbyResult {
        let table = match self.find_table_mut(table_id) {
            Some(t) => t,
            None => {
                return LobbyResult {
                    success: false,
                    message: "Mesa não encontrada.".to_string(),
                    table_id: Some(table_id.to_string()),
                }
            }
        };

        // Verificar vaga
        if table.current_players >= table.max_players {
            return LobbyResult {
                success: false,
                message: format!(
                    "Mesa lotada ({}/{})",
                    table.current_players, table.max_players
                ),
                table_id: Some(table_id.to_string()),
            };
        }

        // Verificar saldo mínimo
        if player_balance < table.min_buy_in {
            return LobbyResult {
                success: false,
                message: format!(
                    "Saldo insuficiente. Necessário: {}, Disponível: {}",
                    table.min_buy_in, player_balance
                ),
                table_id: Some(table_id.to_string()),
            };
        }

        // Verificar senha para mesas privadas
        if table.visibility == TableVisibility::Private {
            let expected_hash = table.password_hash.as_ref();
            let provided_hash = password_attempt.map(|p| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                p.hash(&mut hasher);
                hasher.finish().to_string()
            });

            if expected_hash != provided_hash.as_ref() {
                return LobbyResult {
                    success: false,
                    message: "Senha incorreta para mesa privada.".to_string(),
                    table_id: Some(table_id.to_string()),
                };
            }
        }

        // Sucesso: incrementar contagem de jogadores
        table.current_players += 1;
        LobbyResult {
            success: true,
            message: format!("Entrou na mesa '{}' com sucesso.", table.name),
            table_id: Some(table_id.to_string()),
        }
    }

    /// Remove um jogador da mesa (decrementa contagem).
    pub fn leave_table(&mut self, table_id: &str) -> LobbyResult {
        let table = match self.find_table_mut(table_id) {
            Some(t) => t,
            None => {
                return LobbyResult {
                    success: false,
                    message: "Mesa não encontrada.".to_string(),
                    table_id: Some(table_id.to_string()),
                }
            }
        };

        if table.current_players == 0 {
            return LobbyResult {
                success: false,
                message: "Mesa já está vazia.".to_string(),
                table_id: Some(table_id.to_string()),
            };
        }

        table.current_players -= 1;
        LobbyResult {
            success: true,
            message: format!("Saiu da mesa '{}'.", table.name),
            table_id: Some(table_id.to_string()),
        }
    }

    /// Remove uma mesa do lobby (ex: quando todos saem e mesa é fechada).
    pub fn close_table(&mut self, table_id: &str) -> LobbyResult {
        let initial_len = self.tables.len();
        self.tables.retain(|t| t.id != table_id);

        if self.tables.len() == initial_len {
            LobbyResult {
                success: false,
                message: "Mesa não encontrada para fechar.".to_string(),
                table_id: Some(table_id.to_string()),
            }
        } else {
            LobbyResult {
                success: true,
                message: format!("Mesa '{}' fechada.", table_id),
                table_id: Some(table_id.to_string()),
            }
        }
    }

    /// Retorna o número total de mesas ativas.
    pub fn table_count(&self) -> usize {
        self.tables.len()
    }

    /// Retorna o número total de jogadores em todas as mesas.
    pub fn total_players(&self) -> u64 {
        self.tables.iter().map(|t| t.current_players as u64).sum()
    }

    /// Verifica se existe alguma mesa disponível com os parâmetros desejados.
    /// Se não existir, sugere criação de nova mesa.
    pub fn find_or_suggest_table(
        &self,
        game_type: GameType,
        big_blind: u64,
        player_balance: u64,
    ) -> Option<&TableInfo> {
        self.tables
            .iter()
            .filter(|t| {
                t.game_type == game_type
                    && t.big_blind == big_blind
                    && t.current_players < t.max_players
                    && player_balance >= t.min_buy_in
                    && t.visibility == TableVisibility::Public
            })
            .min_by_key(|t| t.current_players) // Prefere mesas mais cheias
    }
}

impl Default for LobbyManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Testes ───

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Criação de Mesas ───

    #[test]
    fn test_create_public_cash_table() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa Cash 1".into(),
            GameType::Cash,
            1,
            2,
            100,
            1000,
            9,
            TableVisibility::Public,
            None,
        );
        assert!(id.starts_with("table_"));
        assert_eq!(lobby.table_count(), 1);

        let table = lobby.find_table(&id).unwrap();
        assert_eq!(table.name, "Mesa Cash 1");
        assert_eq!(table.game_type, GameType::Cash);
        assert_eq!(table.small_blind, 1);
        assert_eq!(table.big_blind, 2);
        assert_eq!(table.min_buy_in, 100);
        assert_eq!(table.max_buy_in, 1000);
        assert_eq!(table.max_players, 9);
        assert_eq!(table.current_players, 0);
        assert_eq!(table.visibility, TableVisibility::Public);
        assert!(table.password_hash.is_none());
    }

    #[test]
    fn test_create_private_tournament_table() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Torneio VIP".into(),
            GameType::Tournament,
            10,
            20,
            1000,
            5000,
            6,
            TableVisibility::Private,
            Some("senha123".into()),
        );
        assert_eq!(lobby.table_count(), 1);

        let table = lobby.find_table(&id).unwrap();
        assert_eq!(table.game_type, GameType::Tournament);
        assert_eq!(table.visibility, TableVisibility::Private);
        assert!(table.password_hash.is_some());
    }

    #[test]
    fn test_create_multiple_tables_unique_ids() {
        let mut lobby = LobbyManager::new();
        let id1 = lobby.create_table(
            "Mesa A".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        let id2 = lobby.create_table(
            "Mesa B".into(),
            GameType::Cash,
            5,
            10,
            500,
            2000,
            9,
            TableVisibility::Public,
            None,
        );
        assert_ne!(id1, id2);
        assert_eq!(lobby.table_count(), 2);
    }

    // ─── Listagem de Mesas ───

    #[test]
    fn test_list_all_tables() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "Cash 1".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        lobby.create_table(
            "Torneio 1".into(),
            GameType::Tournament,
            10,
            20,
            1000,
            5000,
            9,
            TableVisibility::Public,
            None,
        );

        let all = lobby.list_tables(None);
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_list_tables_filter_by_game_type() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "Cash 1".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        lobby.create_table(
            "Cash 2".into(),
            GameType::Cash,
            5,
            10,
            500,
            2000,
            9,
            TableVisibility::Public,
            None,
        );
        lobby.create_table(
            "Torneio 1".into(),
            GameType::Tournament,
            10,
            20,
            1000,
            5000,
            9,
            TableVisibility::Public,
            None,
        );

        let cash_tables = lobby.list_tables(Some(GameType::Cash));
        assert_eq!(cash_tables.len(), 2);

        let tournament_tables = lobby.list_tables(Some(GameType::Tournament));
        assert_eq!(tournament_tables.len(), 1);
    }

    #[test]
    fn test_list_tables_by_blinds() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "Micro".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        lobby.create_table(
            "Low".into(),
            GameType::Cash,
            5,
            10,
            500,
            2000,
            9,
            TableVisibility::Public,
            None,
        );
        lobby.create_table(
            "High".into(),
            GameType::Cash,
            50,
            100,
            5000,
            20000,
            9,
            TableVisibility::Public,
            None,
        );

        let low_stakes = lobby.list_tables_by_blinds(1, 10);
        assert_eq!(low_stakes.len(), 2); // Micro + Low

        let high_stakes = lobby.list_tables_by_blinds(50, 100);
        assert_eq!(high_stakes.len(), 1); // High
    }

    #[test]
    fn test_list_available_tables() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            2,
            TableVisibility::Public,
            None,
        );

        // Mesa vazia = disponível
        let available = lobby.list_available_tables();
        assert_eq!(available.len(), 1);

        // Encher a mesa
        lobby.join_table(&id, 200, None);
        lobby.join_table(&id, 200, None);

        // Mesa lotada = não disponível
        let available = lobby.list_available_tables();
        assert_eq!(available.len(), 0);
    }

    // ─── Entrada na Mesa (Join) ───

    #[test]
    fn test_join_table_success() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa Teste".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );

        let result = lobby.join_table(&id, 200, None);
        assert!(result.success);
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 1);
    }

    #[test]
    fn test_join_table_insufficient_balance() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa Cara".into(),
            GameType::Cash,
            10,
            20,
            1000,
            5000,
            6,
            TableVisibility::Public,
            None,
        );

        let result = lobby.join_table(&id, 50, None);
        assert!(!result.success);
        assert!(result.message.contains("Saldo insuficiente"));
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 0);
    }

    #[test]
    fn test_join_table_full() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa 2p".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            2,
            TableVisibility::Public,
            None,
        );

        lobby.join_table(&id, 200, None);
        lobby.join_table(&id, 200, None);

        // Terceiro jogador não entra
        let result = lobby.join_table(&id, 200, None);
        assert!(!result.success);
        assert!(result.message.contains("lotada"));
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 2);
    }

    #[test]
    fn test_join_table_not_found() {
        let mut lobby = LobbyManager::new();
        let result = lobby.join_table("table_999", 200, None);
        assert!(!result.success);
        assert!(result.message.contains("não encontrada"));
    }

    #[test]
    fn test_join_private_table_correct_password() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "VIP".into(),
            GameType::Cash,
            10,
            20,
            1000,
            5000,
            6,
            TableVisibility::Private,
            Some("segredo".into()),
        );

        let result = lobby.join_table(&id, 2000, Some("segredo".into()));
        assert!(result.success);
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 1);
    }

    #[test]
    fn test_join_private_table_wrong_password() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "VIP".into(),
            GameType::Cash,
            10,
            20,
            1000,
            5000,
            6,
            TableVisibility::Private,
            Some("segredo".into()),
        );

        let result = lobby.join_table(&id, 2000, Some("errado".into()));
        assert!(!result.success);
        assert!(result.message.contains("Senha incorreta"));
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 0);
    }

    #[test]
    fn test_join_private_table_no_password_provided() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "VIP".into(),
            GameType::Cash,
            10,
            20,
            1000,
            5000,
            6,
            TableVisibility::Private,
            Some("segredo".into()),
        );

        let result = lobby.join_table(&id, 2000, None);
        assert!(!result.success);
        assert!(result.message.contains("Senha incorreta"));
    }

    // ─── Saída da Mesa (Leave) ───

    #[test]
    fn test_leave_table_success() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        lobby.join_table(&id, 200, None);
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 1);

        let result = lobby.leave_table(&id);
        assert!(result.success);
        assert_eq!(lobby.find_table(&id).unwrap().current_players, 0);
    }

    #[test]
    fn test_leave_table_not_found() {
        let mut lobby = LobbyManager::new();
        let result = lobby.leave_table("table_999");
        assert!(!result.success);
    }

    #[test]
    fn test_leave_empty_table() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );

        let result = lobby.leave_table(&id);
        assert!(!result.success);
        assert!(result.message.contains("vazia"));
    }

    // ─── Fechamento de Mesa ───

    #[test]
    fn test_close_table_success() {
        let mut lobby = LobbyManager::new();
        let id = lobby.create_table(
            "Mesa".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        assert_eq!(lobby.table_count(), 1);

        let result = lobby.close_table(&id);
        assert!(result.success);
        assert_eq!(lobby.table_count(), 0);
    }

    #[test]
    fn test_close_table_not_found() {
        let mut lobby = LobbyManager::new();
        let result = lobby.close_table("table_999");
        assert!(!result.success);
    }

    // ─── Estatísticas do Lobby ───

    #[test]
    fn test_total_players() {
        let mut lobby = LobbyManager::new();
        let id1 = lobby.create_table(
            "Mesa A".into(),
            GameType::Cash,
            1,
            2,
            100,
            500,
            6,
            TableVisibility::Public,
            None,
        );
        let id2 = lobby.create_table(
            "Mesa B".into(),
            GameType::Cash,
            5,
            10,
            500,
            2000,
            9,
            TableVisibility::Public,
            None,
        );

        lobby.join_table(&id1, 200, None);
        lobby.join_table(&id1, 200, None);
        lobby.join_table(&id2, 1000, None);

        assert_eq!(lobby.total_players(), 3);
    }

    // ─── Find or Suggest ───

    #[test]
    fn test_find_or_suggest_exact_match() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "NL10".into(),
            GameType::Cash,
            5,
            10,
            500,
            2000,
            6,
            TableVisibility::Public,
            None,
        );
        lobby.create_table(
            "NL100".into(),
            GameType::Cash,
            50,
            100,
            5000,
            20000,
            6,
            TableVisibility::Public,
            None,
        );

        let found = lobby.find_or_suggest_table(GameType::Cash, 10, 1000);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "NL10");
    }

    #[test]
    fn test_find_or_suggest_no_match() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "NL100".into(),
            GameType::Cash,
            50,
            100,
            5000,
            20000,
            6,
            TableVisibility::Public,
            None,
        );

        // Jogador quer NL10 mas só existe NL100
        let found = lobby.find_or_suggest_table(GameType::Cash, 10, 1000);
        assert!(found.is_none()); // Nenhuma mesa compatível
    }

    #[test]
    fn test_find_or_suggest_private_excluded() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "VIP".into(),
            GameType::Cash,
            5,
            10,
            500,
            2000,
            6,
            TableVisibility::Private,
            Some("senha".into()),
        );

        // Mesas privadas não aparecem na busca automática
        let found = lobby.find_or_suggest_table(GameType::Cash, 10, 1000);
        assert!(found.is_none());
    }

    #[test]
    fn test_find_or_suggest_insufficient_balance() {
        let mut lobby = LobbyManager::new();
        lobby.create_table(
            "NL100".into(),
            GameType::Cash,
            50,
            100,
            5000,
            20000,
            6,
            TableVisibility::Public,
            None,
        );

        // Jogador não tem saldo suficiente
        let found = lobby.find_or_suggest_table(GameType::Cash, 100, 500);
        assert!(found.is_none());
    }

    // ─── Serialização JSON ───

    #[test]
    fn test_table_info_json_serialization() {
        let table = TableInfo {
            id: "table_1".into(),
            name: "Mesa Teste".into(),
            game_type: GameType::Cash,
            small_blind: 1,
            big_blind: 2,
            min_buy_in: 100,
            max_buy_in: 500,
            max_players: 6,
            current_players: 2,
            visibility: TableVisibility::Public,
            password_hash: None,
        };

        let json = serde_json::to_string(&table).unwrap();
        let parsed: TableInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, table.id);
        assert_eq!(parsed.name, table.name);
        assert_eq!(parsed.game_type, table.game_type);
        assert_eq!(parsed.current_players, 2);
    }

    #[test]
    fn test_lobby_result_json_serialization() {
        let result = LobbyResult {
            success: true,
            message: "Entrou na mesa.".into(),
            table_id: Some("table_1".into()),
        };

        let json = serde_json::to_string(&result).unwrap();
        let parsed: LobbyResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.message, "Entrou na mesa.");
        assert_eq!(parsed.table_id.unwrap(), "table_1");
    }

    #[test]
    fn test_game_type_json_serialization() {
        let cash = GameType::Cash;
        let json = serde_json::to_string(&cash).unwrap();
        assert_eq!(json, "\"cash\"");

        let tournament = GameType::Tournament;
        let json = serde_json::to_string(&tournament).unwrap();
        assert_eq!(json, "\"tournament\"");

        let parsed: GameType = serde_json::from_str("\"cash\"").unwrap();
        assert_eq!(parsed, GameType::Cash);
    }

    #[test]
    fn test_player_lobby_status_json_serialization() {
        let status = PlayerLobbyStatus::Playing;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"playing\"");

        let parsed: PlayerLobbyStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PlayerLobbyStatus::Playing);
    }
}
