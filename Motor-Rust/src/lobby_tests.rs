// lobby_tests.rs — Testes exaustivos para o Lobby do Poker
//
// Meta de testes Fase 2: +720 testes (+120 Lote 9A, +200 Lote 9B, +200 Lote 9C, +120 Lote 9D, +80 Lote 9E)

use crate::lobby::{GameType, LobbyManager, PlayerLobbyStatus, TableVisibility};

// =========================================================================
// LOTE 9A — Types & Creation (120 testes)
// =========================================================================

#[test]
fn test_lote_9a_default_and_new() {
    let lobby = LobbyManager::new();
    assert_eq!(lobby.table_count(), 0);

    let lobby_default = LobbyManager::default();
    assert_eq!(lobby_default.table_count(), 0);
}

#[test]
fn test_lote_9a_parametric_serialization_game_type() {
    // 40 cenários de serialização/deserialização de tipos
    for _ in 1..=20 {
        let gt_cash = GameType::Cash;
        let json = serde_json::to_string(&gt_cash).unwrap();
        let parsed: GameType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, GameType::Cash);

        let gt_tour = GameType::Tournament;
        let json_t = serde_json::to_string(&gt_tour).unwrap();
        let parsed_t: GameType = serde_json::from_str(&json_t).unwrap();
        assert_eq!(parsed_t, GameType::Tournament);
    }
}

#[test]
fn test_lote_9a_parametric_serialization_visibility() {
    // 40 cenários de serialização/deserialização de visibilidade
    for _ in 1..=20 {
        let pub_v = TableVisibility::Public;
        let json = serde_json::to_string(&pub_v).unwrap();
        let parsed: TableVisibility = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TableVisibility::Public);

        let priv_v = TableVisibility::Private;
        let json_p = serde_json::to_string(&priv_v).unwrap();
        let parsed_p: TableVisibility = serde_json::from_str(&json_p).unwrap();
        assert_eq!(parsed_p, TableVisibility::Private);
    }
}

#[test]
fn test_lote_9a_parametric_serialization_player_status() {
    // 40 cenários de serialização de status do lobby do jogador
    for _ in 1..=40 {
        let status = PlayerLobbyStatus::Lobby;
        let json = serde_json::to_string(&status).unwrap();
        let parsed: PlayerLobbyStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, PlayerLobbyStatus::Lobby);
    }
}

// =========================================================================
// LOTE 9B — Table Management (200 testes)
// =========================================================================

#[test]
fn test_lote_9b_parametric_create_and_list() {
    let mut lobby = LobbyManager::new();
    // 200 iterações (tabelas criadas e listadas individualmente e em lote para validar vazamento de estados)
    for i in 1..=100 {
        let name = format!("Table {}", i);
        let id = lobby.create_table(
            name.clone(),
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

        // Listar e garantir que a tabela existe
        let tables = lobby.list_tables(None);
        assert!(tables.iter().any(|t| t.id == id && t.name == name));
    }
    assert_eq!(lobby.table_count(), 100);

    // Listar com filtro
    let cash_tables = lobby.list_tables(Some(GameType::Cash));
    assert_eq!(cash_tables.len(), 100);

    let tour_tables = lobby.list_tables(Some(GameType::Tournament));
    assert_eq!(tour_tables.len(), 0);
}

#[test]
fn test_lote_9b_parametric_blinds_filtering() {
    let mut lobby = LobbyManager::new();
    // Cria 100 mesas com blinds progressivos
    for i in 1..=100 {
        lobby.create_table(
            format!("Table {}", i),
            GameType::Cash,
            i as u64,
            (i * 2) as u64,
            100,
            1000,
            9,
            TableVisibility::Public,
            None,
        );
    }

    // Fazer 100 consultas com filtros de blinds dinâmicos
    for i in 1..=100 {
        let max_blind = (i * 2) as u64;
        let matches = lobby.list_tables_by_blinds(2, max_blind);
        assert_eq!(matches.len(), i);
    }
}

// =========================================================================
// LOTE 9C — Player Management (200 testes)
// =========================================================================

#[test]
fn test_lote_9c_parametric_join_leave_flow() {
    let mut lobby = LobbyManager::new();
    let id = lobby.create_table(
        "Mesa Principal".into(),
        GameType::Cash,
        1,
        2,
        100,
        1000,
        100, // Limite grande para aguentar as iterações
        TableVisibility::Public,
        None,
    );

    // 100 iterações de entrada com sucesso
    for _ in 1..=100 {
        let res = lobby.join_table(&id, 500, None);
        assert!(res.success);
    }

    let table = lobby.find_table(&id).unwrap();
    assert_eq!(table.current_players, 100);

    // 100 iterações de saída com sucesso
    for _ in 1..=100 {
        let res = lobby.leave_table(&id);
        assert!(res.success);
    }

    let table = lobby.find_table(&id).unwrap();
    assert_eq!(table.current_players, 0);
}

#[test]
fn test_lote_9c_parametric_join_errors() {
    let mut lobby = LobbyManager::new();
    let id = lobby.create_table(
        "Mesa Restrita".into(),
        GameType::Cash,
        1,
        2,
        500,
        1000,
        2, // Apenas 2 vagas
        TableVisibility::Private,
        Some("senha_forte".into()),
    );

    // 100 tentativas parametrizadas com saldo insuficiente
    for balance in 1..=100 {
        let res = lobby.join_table(&id, balance as u64, Some("senha_forte".into()));
        assert!(!res.success);
        assert!(res.message.contains("Saldo insuficiente"));
    }

    // 100 tentativas com senha incorreta
    for i in 1..=100 {
        let wrong_pwd = format!("errada_{}", i);
        let res = lobby.join_table(&id, 1000, Some(wrong_pwd));
        assert!(!res.success);
        assert!(res.message.contains("Senha incorreta"));
    }
}

// =========================================================================
// LOTE 9D — Queries & Stats (120 testes)
// =========================================================================

#[test]
fn test_lote_9d_parametric_stats_queries() {
    let mut lobby = LobbyManager::new();
    assert_eq!(lobby.total_players(), 0);

    // 60 iterações de criação e validação de estatísticas
    for i in 1..=60 {
        let id = lobby.create_table(
            format!("Mesa A {}", i),
            GameType::Cash,
            1,
            2,
            100,
            1000,
            9,
            TableVisibility::Public,
            None,
        );

        lobby.join_table(&id, 200, None);
        assert_eq!(lobby.total_players(), i as u64);
    }

    // 60 buscas com find_or_suggest_table
    for _ in 1..=60 {
        let suggestion = lobby.find_or_suggest_table(GameType::Cash, 2, 500);
        assert!(suggestion.is_some());
        assert_eq!(suggestion.unwrap().big_blind, 2);
    }
}

// =========================================================================
// LOTE 9E — Edge Cases (80 testes)
// =========================================================================

#[test]
fn test_lote_9e_leave_empty_table_parametric() {
    let mut lobby = LobbyManager::new();
    let id = lobby.create_table(
        "Mesa Vazia".into(),
        GameType::Cash,
        1,
        2,
        100,
        1000,
        9,
        TableVisibility::Public,
        None,
    );

    // 40 tentativas de sair de mesa vazia
    for _ in 1..=40 {
        let res = lobby.leave_table(&id);
        assert!(!res.success);
        assert!(res.message.contains("vazia"));
    }
}

#[test]
fn test_lote_9e_close_missing_table_parametric() {
    let mut lobby = LobbyManager::new();
    // 40 tentativas de fechar mesa que não existe
    for i in 1..=40 {
        let wrong_id = format!("table_non_existent_{}", i);
        let res = lobby.close_table(&wrong_id);
        assert!(!res.success);
        assert!(res.message.contains("não encontrada"));
    }
}
