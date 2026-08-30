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
        const LEVELS: [(f64, f64, f64); 26] = [
            (25.0, 50.0, 0.0),
            (50.0, 100.0, 0.0),
            (75.0, 150.0, 0.0),
            (100.0, 200.0, 0.0),
            (150.0, 300.0, 0.0),
            (200.0, 400.0, 0.0),
            (300.0, 600.0, 0.0),
            (400.0, 800.0, 0.0),
            (500.0, 1_000.0, 50.0),
            (600.0, 1_200.0, 100.0),
            (800.0, 1_600.0, 200.0),
            (1_000.0, 2_000.0, 300.0),
            (1_200.0, 2_400.0, 400.0),
            (1_500.0, 3_000.0, 500.0),
            (2_000.0, 4_000.0, 500.0),
            (2_500.0, 5_000.0, 700.0),
            (3_000.0, 6_000.0, 800.0),
            (4_000.0, 8_000.0, 1_000.0),
            (5_000.0, 10_000.0, 1_500.0),
            (6_000.0, 12_000.0, 2_000.0),
            (8_000.0, 16_000.0, 2_500.0),
            (10_000.0, 20_000.0, 3_000.0),
            (12_000.0, 24_000.0, 4_000.0),
            (15_000.0, 30_000.0, 5_000.0),
            (20_000.0, 40_000.0, 6_000.0),
            (25_000.0, 50_000.0, 8_000.0),
        ];

        Self {
            name: "Standard Regular (10m)".into(),
            levels: LEVELS
                .into_iter()
                .enumerate()
                .map(|(index, (small_blind, big_blind, ante))| BlindLevel {
                    level_number: (index + 1) as u32,
                    small_blind,
                    big_blind,
                    ante,
                    duration_seconds: 600,
                })
                .collect(),
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
