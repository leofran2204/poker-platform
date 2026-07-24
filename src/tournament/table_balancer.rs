use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableStateSummary {
    pub table_id: String,
    pub active_player_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TableMove {
    pub player_id: String,
    pub from_table_id: String,
    pub to_table_id: String,
}

pub struct TableBalancer;

impl TableBalancer {
    /// Avalia o equilíbrio das mesas ativas e gera os movimentos necessários.
    /// Garante que a diferença de jogadores entre a mesa mais populosa e a menos populosa seja <= 1.
    pub fn balance_tables(tables: &[TableStateSummary]) -> Vec<TableMove> {
        if tables.len() <= 1 {
            return Vec::new();
        }

        let mut current_tables = tables.to_vec();
        let mut moves = Vec::new();

        loop {
            // Encontrar mesa com maior número de jogadores e mesa com menor número
            let mut min_idx = 0;
            let mut max_idx = 0;

            for i in 1..current_tables.len() {
                if current_tables[i].active_player_ids.len() < current_tables[min_idx].active_player_ids.len() {
                    min_idx = i;
                }
                if current_tables[i].active_player_ids.len() > current_tables[max_idx].active_player_ids.len() {
                    max_idx = i;
                }
            }

            let max_count = current_tables[max_idx].active_player_ids.len();
            let min_count = current_tables[min_idx].active_player_ids.len();

            // Se a diferença for <= 1, as mesas estão perfeitamente balanceadas
            if max_count.saturating_sub(min_count) <= 1 {
                break;
            }

            // Mover 1 jogador da mesa mais populosa para a menos populosa
            if let Some(moved_player) = current_tables[max_idx].active_player_ids.pop() {
                current_tables[min_idx].active_player_ids.push(moved_player.clone());

                moves.push(TableMove {
                    player_id: moved_player,
                    from_table_id: current_tables[max_idx].table_id.clone(),
                    to_table_id: current_tables[min_idx].table_id.clone(),
                });
            } else {
                break;
            }
        }

        moves
    }
}
