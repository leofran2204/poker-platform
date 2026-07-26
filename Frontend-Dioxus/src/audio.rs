//! audio.rs — Gerenciador e sintetizador de áudio WebAssembly (Web Audio API)
//!
//! Fornece efeitos sonoros instantâneos e ultraleves para ações da mesa
//! sem necessidade de carregamento de arquivos externos de áudio.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEvent {
    /// Som de aposta / fichas caindo no pote
    ChipBet,
    /// Som de descarte de cartas (fold)
    Fold,
    /// Som de dar mesa / check
    Check,
    /// Som de vitória no showdown (vinheta triádica)
    Win,
}

pub struct SoundManager;

impl SoundManager {
    /// Toca um evento sonoro sintetizado utilizando Web Audio API
    pub fn play(event: SoundEvent) {
        #[cfg(target_arch = "wasm32")]
        {
            use web_sys::{AudioContext, OscillatorType};

            let ctx = match AudioContext::new() {
                Ok(c) => c,
                Err(_) => return,
            };

            let now = ctx.current_time();

            match event {
                SoundEvent::ChipBet => {
                    // Tom metálico duplo de fichas
                    if let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                        osc.set_type(OscillatorType::Sine);
                        osc.frequency().set_value_at_time(800.0, now).ok();
                        osc.frequency().exponential_ramp_to_value_at_time(1200.0, now + 0.05).ok();
                        gain.gain().set_value_at_time(0.3, now).ok();
                        gain.gain().exponential_ramp_to_value_at_time(0.01, now + 0.08).ok();
                        osc.connect_with_audio_node(&gain).ok();
                        gain.connect_with_audio_node(&ctx.destination()).ok();
                        osc.start_with_when(now).ok();
                        osc.stop_with_when(now + 0.08).ok();
                    }
                }
                SoundEvent::Fold => {
                    // Tom grave descendente curto (fold)
                    if let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                        osc.set_type(OscillatorType::Triangle);
                        osc.frequency().set_value_at_time(350.0, now).ok();
                        osc.frequency().exponential_ramp_to_value_at_time(150.0, now + 0.12).ok();
                        gain.gain().set_value_at_time(0.25, now).ok();
                        gain.gain().exponential_ramp_to_value_at_time(0.01, now + 0.12).ok();
                        osc.connect_with_audio_node(&gain).ok();
                        gain.connect_with_audio_node(&ctx.destination()).ok();
                        osc.start_with_when(now).ok();
                        osc.stop_with_when(now + 0.12).ok();
                    }
                }
                SoundEvent::Check => {
                    // Bater duplo na mesa
                    if let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                        osc.set_type(OscillatorType::Sine);
                        osc.frequency().set_value_at_time(220.0, now).ok();
                        gain.gain().set_value_at_time(0.2, now).ok();
                        gain.gain().exponential_ramp_to_value_at_time(0.01, now + 0.05).ok();
                        osc.connect_with_audio_node(&gain).ok();
                        gain.connect_with_audio_node(&ctx.destination()).ok();
                        osc.start_with_when(now).ok();
                        osc.stop_with_when(now + 0.05).ok();
                    }
                }
                SoundEvent::Win => {
                    // Acorde triádico harmônico de vitória (C - E - G)
                    let freqs = [523.25, 659.25, 783.99]; // C5, E5, G5
                    for (i, &freq) in freqs.iter().enumerate() {
                        let offset = i as f64 * 0.08;
                        if let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                            osc.set_type(OscillatorType::Sine);
                            osc.frequency().set_value_at_time(freq, now + offset).ok();
                            gain.gain().set_value_at_time(0.3, now + offset).ok();
                            gain.gain().exponential_ramp_to_value_at_time(0.01, now + offset + 0.3).ok();
                            osc.connect_with_audio_node(&gain).ok();
                            gain.connect_with_audio_node(&ctx.destination()).ok();
                            osc.start_with_when(now + offset).ok();
                            osc.stop_with_when(now + offset + 0.3).ok();
                        }
                    }
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Em testes nativos fora do WASM, apenas registra o evento sem pânico
            log::info!("[SoundManager Play]: {:?}", event);
        }
    }
}
