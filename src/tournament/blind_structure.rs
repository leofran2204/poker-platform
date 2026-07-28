use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlindLevel {
    pub level_number: u32,
    pub small_blind: f64,
    pub big_blind: f64,
    pub ante: f64,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlindStructure {
    pub name: String,
    pub levels: Vec<BlindLevel>,
}

impl BlindStructure {
    /// Estrutura padrão de Blinds para Torneio Regular (níveis de 10 minutos).
    pub fn standard_regular() -> Self {
        Self {
            name: "Standard Regular (10m)".into(),
            levels: vec![
                BlindLevel {
                    level_number: 1,
                    small_blind: 10.0,
                    big_blind: 20.0,
                    ante: 0.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 2,
                    small_blind: 15.0,
                    big_blind: 30.0,
                    ante: 0.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 3,
                    small_blind: 25.0,
                    big_blind: 50.0,
                    ante: 0.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 4,
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: 10.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 5,
                    small_blind: 75.0,
                    big_blind: 150.0,
                    ante: 15.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 6,
                    small_blind: 100.0,
                    big_blind: 200.0,
                    ante: 25.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 7,
                    small_blind: 150.0,
                    big_blind: 300.0,
                    ante: 40.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 8,
                    small_blind: 200.0,
                    big_blind: 400.0,
                    ante: 50.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 9,
                    small_blind: 300.0,
                    big_blind: 600.0,
                    ante: 75.0,
                    duration_seconds: 600,
                },
                BlindLevel {
                    level_number: 10,
                    small_blind: 500.0,
                    big_blind: 1000.0,
                    ante: 100.0,
                    duration_seconds: 600,
                },
            ],
        }
    }

    /// Estrutura de Blinds Turbo (níveis de 3 minutos).
    pub fn turbo_fast() -> Self {
        let mut structure = Self::standard_regular();
        structure.name = "Turbo Fast (3m)".into();
        for level in &mut structure.levels {
            level.duration_seconds = 180;
        }
        structure
    }

    pub fn get_level(&self, level_idx: usize) -> Option<&BlindLevel> {
        self.levels.get(level_idx)
    }
}
