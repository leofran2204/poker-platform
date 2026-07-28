//! Módulo de Codec Binário para transmissão de alta performance via WebSockets.
//!
//! Substitui ou complementa o JSON de texto com um formato binário compacto (Tag + Payload)
//! reduzindo a latência do WebSocket e prevenindo inspection/tampering.

use serde::{Deserialize, Serialize};

const MAX_BINARY_PAYLOAD_BYTES: usize = 64 * 1024;

/// Tipos de mensagens binárias do protocolo de jogo.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOpcode {
    Ping = 0x01,
    Pong = 0x02,
    PlayerAction = 0x10,
    GameStateUpdate = 0x20,
    ProvablyFairHandStart = 0x30,
    ProvablyFairHandEnd = 0x31,
    UnknownError = 0xFF,
}

impl TryFrom<u8> for BinaryOpcode {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, <Self as TryFrom<u8>>::Error> {
        match value {
            0x01 => Ok(BinaryOpcode::Ping),
            0x02 => Ok(BinaryOpcode::Pong),
            0x10 => Ok(BinaryOpcode::PlayerAction),
            0x20 => Ok(BinaryOpcode::GameStateUpdate),
            0x30 => Ok(BinaryOpcode::ProvablyFairHandStart),
            0x31 => Ok(BinaryOpcode::ProvablyFairHandEnd),
            0xFF => Ok(BinaryOpcode::UnknownError),
            other => Err(format!("Opcode binário desconhecido: 0x{other:02X}")),
        }
    }
}

/// Mensagem Binária do Protocolo de Poker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryPacket {
    pub opcode: u8,
    pub payload: Vec<u8>,
}

impl BinaryPacket {
    pub fn new(opcode: BinaryOpcode, payload: Vec<u8>) -> Self {
        Self {
            opcode: opcode as u8,
            payload,
        }
    }

    /// Codifica o pacote binário: [Opcode (1 byte)] + [Payload Length (4 bytes u32 BE)] + [Payload Bytes].
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1 + 4 + self.payload.len());
        buf.push(self.opcode);
        buf.extend_from_slice(&(self.payload.len() as u32).to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }

    /// Decodifica um buffer de bytes para a estrutura BinaryPacket.
    pub fn decode(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 5 {
            return Err(
                "Buffer muito curto para conter cabeçalho binário (mínimo 5 bytes)".to_string(),
            );
        }

        let opcode = buf[0];
        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&buf[1..5]);
        let payload_len = u32::from_be_bytes(len_bytes) as usize;

        if payload_len > MAX_BINARY_PAYLOAD_BYTES {
            return Err(format!(
                "Payload binário excede o limite de {MAX_BINARY_PAYLOAD_BYTES} bytes"
            ));
        }

        if buf.len() != 5 + payload_len {
            return Err(format!(
                "Payload binário inválido: esperado {payload_len} bytes, recebido {}",
                buf.len() - 5
            ));
        }

        let payload = buf[5..5 + payload_len].to_vec();

        Ok(Self { opcode, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_packet_encode_decode() {
        let original_payload = b"fold_hand_123".to_vec();
        let packet = BinaryPacket::new(BinaryOpcode::PlayerAction, original_payload.clone());

        let encoded = packet.encode();
        assert_eq!(encoded[0], 0x10);

        let decoded = BinaryPacket::decode(&encoded).unwrap();
        assert_eq!(decoded.opcode, BinaryOpcode::PlayerAction as u8);
        assert_eq!(decoded.payload, original_payload);
    }

    #[test]
    fn test_binary_packet_short_buffer() {
        let invalid_buf = vec![0x01, 0x00];
        let result = BinaryPacket::decode(&invalid_buf);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_trailing_or_oversized_binary_payloads() {
        let trailing = vec![BinaryOpcode::Ping as u8, 0, 0, 0, 0, 99];
        assert!(BinaryPacket::decode(&trailing).is_err());

        let oversized = [BinaryOpcode::PlayerAction as u8, 0, 1, 0, 1];
        assert!(BinaryPacket::decode(&oversized).is_err());
    }
}

// ─── Proptests & Fuzzing Massivo de Pacotes Binários ───

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        #[test]
        fn proptest_binary_packet_roundtrip(
            opcode_byte in any::<u8>(),
            payload in proptest::collection::vec(any::<u8>(), 0..2048)
        ) {
            let packet = BinaryPacket {
                opcode: opcode_byte,
                payload: payload.clone(),
            };

            let encoded = packet.encode();
            let decoded = BinaryPacket::decode(&encoded).unwrap();

            prop_assert_eq!(decoded.opcode, opcode_byte);
            prop_assert_eq!(decoded.payload, payload);
        }

        #[test]
        fn proptest_binary_packet_fuzz_random_bytes_never_panics(
            random_buf in proptest::collection::vec(any::<u8>(), 0..4096)
        ) {
            // NUNCA deve panicar, independente do lixo binário recebido no socket
            let _ = BinaryPacket::decode(&random_buf);
        }
    }
}
