"""Efeitos sonoros e trilha ambiente originais, sintetizados em código (sem licença de terceiros).

Gera WAV 48 kHz em public/sfx/. Determinístico: a mesma semente produz os mesmos
arquivos. A trilha é um pad calmo em lá menor, pensado para ficar baixo sob a voz.

Uso: python sound.py
"""
from __future__ import annotations

import wave
from pathlib import Path

import numpy as np

RATE = 48_000
OUT = Path(__file__).resolve().parent / "public" / "sfx"
RNG = np.random.default_rng(20261108)


def t_axis(seconds: float) -> np.ndarray:
    return np.arange(int(seconds * RATE)) / RATE


def lowpass(signal: np.ndarray, cutoff: np.ndarray | float) -> np.ndarray:
    """Passa-baixa de um polo, com corte fixo ou variando no tempo.

    Corte fixo: resposta do mesmo filtro aplicada no domínio da frequência (rápido
    para minutos de áudio). Corte variável: recursão amostra a amostra (sons curtos).
    """
    if np.ndim(cutoff) == 0:
        alpha = 1 - np.exp(-2 * np.pi * float(cutoff) / RATE)
        pad = 4096  # a resposta ao impulso decai muito antes disso
        size = 1 << int(np.ceil(np.log2(signal.size + pad)))
        freqs = np.fft.rfftfreq(size)
        response = alpha / (1 - (1 - alpha) * np.exp(-2j * np.pi * freqs))
        return np.fft.irfft(np.fft.rfft(signal, size) * response, size)[: signal.size]
    cutoff = np.broadcast_to(np.asarray(cutoff, dtype=float), signal.shape)
    alpha = 1 - np.exp(-2 * np.pi * cutoff / RATE)
    out = np.empty_like(signal)
    state = 0.0
    for i, (x, a) in enumerate(zip(signal, alpha)):
        state += a * (x - state)
        out[i] = state
    return out


def highpass(signal: np.ndarray, cutoff: float) -> np.ndarray:
    return signal - lowpass(signal, cutoff)


def write(name: str, data: np.ndarray, peak: float = 0.8) -> None:
    if data.ndim == 1:
        data = data[:, None]
    maximum = np.max(np.abs(data)) or 1.0
    scaled = np.clip(data / maximum * peak, -1, 1)
    pcm = (scaled * 32767).astype("<i2")
    OUT.mkdir(parents=True, exist_ok=True)
    with wave.open(str(OUT / f"{name}.wav"), "wb") as handle:
        handle.setnchannels(pcm.shape[1])
        handle.setsampwidth(2)
        handle.setframerate(RATE)
        handle.writeframes(pcm.tobytes())


def deal() -> np.ndarray:
    """Carta deslizando no feltro: ruído filtrado com ataque rápido."""
    t = t_axis(0.24)
    noise = RNG.standard_normal(t.size)
    body = highpass(lowpass(noise, 6500), 900)
    envelope = np.minimum(t / 0.012, 1) * np.exp(-t / 0.07)
    return body * envelope


def flip() -> np.ndarray:
    """Virada de carta: estalo curto de papel com um leve corpo grave."""
    t = t_axis(0.09)
    snap = highpass(RNG.standard_normal(t.size), 2500) * np.exp(-t / 0.006)
    thump = np.sin(2 * np.pi * 190 * t) * np.exp(-t / 0.02) * 0.35
    return snap + thump


def chip_hit(pitch: float = 1.0) -> np.ndarray:
    """Ficha de cerâmica: parciais inarmônicos curtos e um transiente."""
    t = t_axis(0.14)
    tone = sum(
        amp * np.sin(2 * np.pi * freq * pitch * t) * np.exp(-t / decay)
        for freq, amp, decay in [(2950, 1.0, 0.018), (4420, 0.6, 0.012), (6230, 0.35, 0.008)]
    )
    click = highpass(RNG.standard_normal(t.size), 3000) * np.exp(-t / 0.0015) * 0.6
    return tone + click


def chips() -> np.ndarray:
    """Pequena pilha de fichas assentando no pote."""
    out = np.zeros(int(0.32 * RATE))
    for offset, gain in [(0.0, 1.0), (0.034, 0.7), (0.061, 0.85), (0.097, 0.55), (0.13, 0.4)]:
        hit = chip_hit(1 + RNG.uniform(-0.06, 0.06)) * gain
        start = int(offset * RATE)
        out[start:start + hit.size] += hit[: out.size - start]
    return out


def whoosh() -> np.ndarray:
    """Passagem suave para trocas de cena."""
    t = t_axis(0.7)
    shape = np.sin(np.pi * t / t[-1]) ** 2
    cutoff = 300 + 2600 * shape
    return lowpass(RNG.standard_normal(t.size), cutoff) * shape


def pop() -> np.ndarray:
    """Destaque de interface: tom curto subindo de afinação."""
    t = t_axis(0.16)
    freq = 520 + 300 * np.minimum(t / 0.05, 1)
    phase = 2 * np.pi * np.cumsum(freq) / RATE
    return np.sin(phase) * np.minimum(t / 0.004, 1) * np.exp(-t / 0.05)


def stamp() -> np.ndarray:
    """Carimbo de decisão: impacto grave e estalo."""
    t = t_axis(0.4)
    low = np.sin(2 * np.pi * 85 * t) * np.exp(-t / 0.09)
    knock = highpass(RNG.standard_normal(t.size), 1200) * np.exp(-t / 0.01) * 0.5
    return low + knock


def reverb(signal: np.ndarray, seconds: float = 3.2, block: int = 1 << 16) -> np.ndarray:
    """Reverberação por convolução com cauda de ruído (overlap-add em blocos)."""
    tail = t_axis(seconds)
    impulse = lowpass(RNG.standard_normal(tail.size), 5200) * np.exp(-tail / 0.9)
    impulse /= np.sqrt(np.sum(impulse ** 2))
    size = 1 << int(np.ceil(np.log2(block + impulse.size)))
    spectrum = np.fft.rfft(impulse, size)
    out = np.zeros(signal.size + impulse.size)
    for start in range(0, signal.size, block):
        chunk = signal[start:start + block]
        result = np.fft.irfft(np.fft.rfft(chunk, size) * spectrum, size)
        end = min(out.size, start + size)
        out[start:end] += result[: end - start]
    return out[: signal.size]


def note(freq: float, t: np.ndarray, detune: float, lfo_phase: float) -> np.ndarray:
    """Timbre quente por síntese aditiva (sem aliasing), com leve chorus.

    Os harmônicos saem das potências de e^{iθ} (sin nθ = Im z^n): uma exponencial
    por nota em vez de um seno por harmônico.
    """
    drift = 1 + detune * np.sin(2 * np.pi * 0.11 * t + lfo_phase)
    base = np.exp(1j * 2 * np.pi * freq * drift * t)
    power = base.copy()
    voice = np.zeros_like(t)
    for n in range(1, 8):
        voice += power.imag / n ** 1.7
        power *= base
    return voice


def music(seconds: float = 300.0) -> np.ndarray:
    """Pad ambiente: Am9 – Fmaj9 – Cmaj7 – G6, 8 s por acorde, estéreo."""
    hz = lambda midi: 440 * 2 ** ((midi - 69) / 12)  # noqa: E731
    chords = [
        [45, 57, 60, 64, 67, 71],  # Am9
        [41, 57, 60, 64, 67, 69],  # Fmaj9 (com a nona soando como G no topo)
        [48, 55, 59, 64, 67, 72],  # Cmaj7
        [43, 55, 59, 62, 64, 67],  # G6
    ]
    t = t_axis(seconds)
    span = 8.0
    left = np.zeros_like(t)
    right = np.zeros_like(t)
    for index in range(int(np.ceil(seconds / span)) + 1):
        start = index * span - 2.0
        window = (t >= start) & (t < start + span + 4.0)
        if not window.any():
            continue
        local = t[window] - start
        envelope = np.sin(np.pi * np.clip(local / (span + 4.0), 0, 1)) ** 2
        chord = chords[index % len(chords)]
        for k, midi in enumerate(chord):
            gain = 0.55 if k == 0 else 0.22
            freq = hz(midi - 12) if k == 0 else hz(midi)
            left[window] += gain * envelope * note(freq, local, 0.0016, k * 0.7)
            right[window] += gain * envelope * note(freq, local, 0.0019, k * 0.7 + 1.3)
    # Brilho discreto: notas altas da escala com trêmulo lento.
    shimmer_notes = [76, 79, 83, 81]
    for index in range(int(seconds // 4)):
        start = index * 4.0 + 1.0
        window = (t >= start) & (t < start + 5.0)
        local = t[window] - start
        tone = np.sin(2 * np.pi * hz(shimmer_notes[index % 4]) * local)
        env = np.sin(np.pi * np.clip(local / 5.0, 0, 1)) ** 2 * (0.5 + 0.5 * np.sin(2 * np.pi * 0.5 * local))
        pan = 0.5 + 0.4 * np.sin(index)
        left[window] += 0.05 * tone * env * (1 - pan)
        right[window] += 0.05 * tone * env * pan
    left = lowpass(left, 2400)
    right = lowpass(right, 2400)
    wet_l, wet_r = reverb(left), reverb(right)
    fade = np.minimum(t / 3.0, 1) * np.minimum((seconds - t) / 4.0, 1)
    return np.stack([(0.55 * left + 0.8 * wet_l) * fade, (0.55 * right + 0.8 * wet_r) * fade], axis=1)


if __name__ == "__main__":
    for name, maker in [("deal", deal), ("flip", flip), ("chips", chips), ("chip", chip_hit),
                        ("whoosh", whoosh), ("pop", pop), ("stamp", stamp)]:
        write(name, maker())
        print(f"sfx: {name}")
    write("music", music(), peak=0.7)
    print("sfx: music")
