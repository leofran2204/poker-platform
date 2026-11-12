"""Narração dos vídeos em Remotion: voz neural da Microsoft, tempos por palavra e cache.

Motores:
- `azure` (padrão quando AZURE_SPEECH_KEY existe): Azure AI Speech, plano gratuito F0,
  com contador mensal que para antes da cota de 500 mil caracteres;
- `edge`: serviço gratuito do Edge (edge-tts), sem conta, como alternativa.

Fala idêntica já sintetizada em outro vídeo é copiada (mesma tomada, sem gastar cota).
Reaproveita de `render.py` o glossário de pronúncia (só na síntese) e a projeção dos
tempos reais da voz de volta às palavras escritas, para legendas e animações.

Uso (Python do ambiente editorial: edge-tts 7.3.1 e azure-cognitiveservices-speech 1.44.0):
    python narrate.py src/data/pineapple-draw-rules.json src/data/academy-m5l3.json
    python narrate.py --engine edge src/data/pineapple-draw-trailer.json
"""
from __future__ import annotations

import argparse
import asyncio
import datetime as dt
import hashlib
import json
import os
import shutil
import subprocess
import sys
import wave
from pathlib import Path
from xml.sax.saxutils import escape

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))
import render  # noqa: E402  (glossário e projeção de tempos já revisados)

PUBLIC = HERE / "public"
GENERATED = HERE / "src" / "generated"
USAGE = PUBLIC / "narration" / "azure-usage.json"
TICKS_PER_MS = 10_000  # offsets em unidades de 100 ns (Edge e Azure)
# Cota gratuita: 500 mil caracteres neurais por mês. Parar bem antes do limite.
MONTHLY_BUDGET = 400_000


def user_env(name: str) -> str | None:
    """Variável de ambiente, incluindo as gravadas no perfil do usuário depois do login."""
    if os.environ.get(name):
        return os.environ[name]
    if sys.platform == "win32":
        import winreg

        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, "Environment") as key:
                return winreg.QueryValueEx(key, name)[0]
        except OSError:
            return None
    return None


def wav_duration_ms(path: Path) -> int:
    with wave.open(str(path), "rb") as handle:
        return round(handle.getnframes() * 1000 / handle.getframerate())


def charge(characters: int) -> None:
    """Soma o uso do mês e recusa a síntese se ultrapassar o orçamento seguro."""
    month = dt.date.today().strftime("%Y-%m")
    usage = json.loads(USAGE.read_text(encoding="utf-8")) if USAGE.exists() else {}
    total = usage.get(month, 0) + characters
    if total > MONTHLY_BUDGET:
        raise SystemExit(
            f"Orçamento mensal de narração atingido ({total} > {MONTHLY_BUDGET} caracteres). "
            "Aguarde o próximo mês ou revise a cota antes de continuar."
        )
    usage[month] = total
    USAGE.parent.mkdir(parents=True, exist_ok=True)
    USAGE.write_text(json.dumps(usage, indent=1), encoding="utf-8")


async def synthesize_edge(spoken: str, voice: str, rate: str, mp3: Path) -> list[dict]:
    import edge_tts

    words: list[dict] = []
    tts = edge_tts.Communicate(spoken, voice, rate=rate, boundary="WordBoundary")
    with mp3.open("wb") as stream:
        async for event in tts.stream():
            if event["type"] == "audio":
                stream.write(event["data"])
            elif event["type"] == "WordBoundary":
                words.append(event)
    return words


def synthesize_azure(spoken: str, voice: str, rate: str, wav: Path, body: str | None = None) -> list[dict]:
    import azure.cognitiveservices.speech as speechsdk

    key, region = user_env("AZURE_SPEECH_KEY"), user_env("AZURE_SPEECH_REGION")
    if not key or not region:
        raise SystemExit("AZURE_SPEECH_KEY/AZURE_SPEECH_REGION ausentes; use --engine edge")
    ssml = (
        '<speak version="1.0" xmlns="http://www.w3.org/2001/10/synthesis" xml:lang="pt-BR">'
        f'<voice name="{voice}"><prosody rate="{rate}">{body or escape(spoken)}</prosody></voice></speak>'
    )
    charge(len(spoken))
    config = speechsdk.SpeechConfig(subscription=key, region=region)
    config.set_speech_synthesis_output_format(speechsdk.SpeechSynthesisOutputFormat.Riff48Khz16BitMonoPcm)
    config.set_property(speechsdk.PropertyId.SpeechServiceResponse_RequestWordBoundary, "true")
    synthesizer = speechsdk.SpeechSynthesizer(
        speech_config=config, audio_config=speechsdk.audio.AudioOutputConfig(filename=str(wav))
    )
    words: list[dict] = []

    def on_boundary(event) -> None:
        if event.boundary_type == speechsdk.SpeechSynthesisBoundaryType.Word:
            words.append({
                "text": event.text,
                "offset": event.audio_offset,
                "duration": round(event.duration.total_seconds() * 10_000_000),
            })

    synthesizer.synthesis_word_boundary.connect(on_boundary)
    result = synthesizer.speak_ssml_async(ssml).get()
    if result.reason != speechsdk.ResultReason.SynthesizingAudioCompleted:
        details = result.cancellation_details
        raise RuntimeError(f"Azure: {details.reason} {details.error_details}")
    return words


async def synthesize(text: str, voice: str, rate: str, target: Path, engine: str) -> list[dict]:
    """Gera `target` (.wav 48 kHz mono) e devolve as palavras canônicas com tempos."""
    # Azure: termos em inglês lidos em inglês (<lang en-US>); Edge não aceita SSML próprio.
    english = engine == "azure"
    spoken, _ = render.pronunciation_plan(text, english)
    for attempt in range(4):
        try:
            if english:
                words = synthesize_azure(spoken, voice, rate, target, render.speech_ssml(text))
            else:
                mp3 = target.with_suffix(".partial.mp3")
                words = await synthesize_edge(spoken, voice, rate, mp3)
                subprocess.run(
                    [render.FFMPEG, "-y", "-loglevel", "error", "-i", str(mp3), "-ar", "48000", "-ac", "1", str(target)],
                    check=True,
                )
                mp3.unlink(missing_ok=True)
            if not render.complete_speech(spoken, words):
                raise RuntimeError(f"Narração incompleta: {target.name}")
            return render.canonical_boundaries(text, words, english)
        except SystemExit:
            raise
        except Exception:
            if attempt == 3:
                raise
            await asyncio.sleep(1 + attempt)
    raise AssertionError("inalcançável")


async def narrate(script_path: Path, engine: str) -> Path:
    script = json.loads(script_path.read_text(encoding="utf-8"))
    video, voice, rate = script["id"], script["voice"], script.get("rate", "+0%")
    folder = PUBLIC / "narration" / video
    folder.mkdir(parents=True, exist_ok=True)
    GENERATED.mkdir(parents=True, exist_ok=True)
    scenes = []
    for scene in script["scenes"]:
        english = engine == "azure"
        spoken, _ = render.pronunciation_plan(scene["voice"], english)
        profile = render.MOTION_PRONUNCIATION_VERSION if english else render.PRONUNCIATION_VERSION
        key = json.dumps([engine, voice, rate, spoken, profile], ensure_ascii=False)
        digest = hashlib.sha256(key.encode()).hexdigest()[:16]
        audio = folder / f"{scene['id']}-{digest}.wav"
        bounds = audio.with_suffix(".json")
        if not (audio.exists() and bounds.exists()):
            # Mesma fala já sintetizada em outro vídeo: reaproveita a mesma tomada (voz idêntica, sem gastar cota).
            for twin in sorted((PUBLIC / "narration").glob(f"*/*-{digest}.wav")):
                if twin.with_suffix(".json").exists():
                    shutil.copyfile(twin, audio)
                    shutil.copyfile(twin.with_suffix(".json"), bounds)
                    break
        if audio.exists() and bounds.exists():
            words = json.loads(bounds.read_text(encoding="utf-8"))
        else:
            words = await synthesize(scene["voice"], voice, rate, audio, engine)
            bounds.write_text(json.dumps(words, ensure_ascii=False), encoding="utf-8")
        scenes.append({
            "id": scene["id"],
            "audio": audio.relative_to(PUBLIC).as_posix(),
            "durationMs": wav_duration_ms(audio),
            "words": [
                {
                    "text": word["text"],
                    "startMs": round(word["offset"] / TICKS_PER_MS),
                    "endMs": round((word["offset"] + word["duration"]) / TICKS_PER_MS),
                }
                for word in words
            ],
        })
        print(f"{video}/{scene['id']}: {scenes[-1]['durationMs'] / 1000:.2f}s")
    output = GENERATED / f"{video}.timings.json"
    output.write_text(
        json.dumps({"voice": voice, "rate": rate, "engine": engine, "scenes": scenes}, ensure_ascii=False, indent=1),
        encoding="utf-8",
    )
    return output


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("scripts", nargs="+", type=Path)
    parser.add_argument("--engine", choices=["azure", "edge"], default=None)
    args = parser.parse_args()
    chosen = args.engine or ("azure" if user_env("AZURE_SPEECH_KEY") else "edge")
    for path in args.scripts:
        print(asyncio.run(narrate(path, chosen)))
