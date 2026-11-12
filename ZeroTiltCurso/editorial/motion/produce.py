"""Produção final dos vídeos em Remotion: render, loudness, legendas, capítulos e capa.

Pré-requisitos: `narrate.py` e `sound.py` já executados; dependências npm instaladas.
Saída mestre (1080p, fora do git): artifacts/video-pilot/<id>/.
Com `--publish`, também grava a versão web (1080p, como as demais aulas do site) em Frontend-Web/public/videos/, o
manifesto e a transcrição em ZeroTiltCurso/editorial/ e o bloco de vídeo do site:
- roteiro com "lesson" (aulas da Academy) → courseContent.json da aula;
- demais roteiros (filme de regras, trailer) → src/data/pineappleDrawFilms.json.
`render.py --check` confere tudo de novo a partir dos arquivos versionados.

Uso:
    python produce.py PineappleDrawRules src/data/pineapple-draw-rules.json --poster 31 --publish
    python produce.py AcademyM5L3 src/data/academy-m5l3.json --publish
    python produce.py AcademyM2L2 src/data/m2l2.json          (piloto, sem publicar)
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PILOT = ROOT / "artifacts" / "video-pilot"
EDITORIAL = HERE.parent
FRONT = ROOT / "Frontend-Web"
VIDEOS = FRONT / "public" / "videos"
COURSE_FILE = FRONT / "src" / "data" / "courseContent.json"
TRAINING_FILE = FRONT / "src" / "data" / "courseTraining.json"
FILMS_FILE = FRONT / "src" / "data" / "pineappleDrawFilms.json"
NODE = os.environ.get(
    "ZT_NODE",
    str(Path.home() / ".cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe"),
)
REMOTION = HERE / "node_modules/@remotion/cli/remotion-cli.js"
LEAD, TAIL = 0.45, 0.65  # iguais a src/lib/timeline.ts
# Versão do pipeline Remotion publicado (troca de cena, legenda ou codificação web → +1).
MOTION_RENDER_VERSION = 2
WEB_CRF = 26
WEB_AUDIO = "128k"

sys.path.insert(0, str(EDITORIAL))
import render  # noqa: E402  (glossário, impressões digitais e checagem compartilhados)


def js_round(value: float) -> int:
    """Math.round do JavaScript (meio para cima), para casar quadro a quadro com timeline.ts."""
    return math.floor(value + 0.5)


def plan(script: dict, timings: dict) -> list[dict]:
    fps, cursor, scenes = script["fps"], 0, []
    for scene in script["scenes"]:
        timing = next(item for item in timings["scenes"] if item["id"] == scene["id"])
        audio_at = js_round(LEAD * fps)
        duration = audio_at + js_round(timing["durationMs"] / 1000 * fps) + js_round(scene.get("padAfter", TAIL) * fps)
        scenes.append({**scene, "from": cursor, "audioAt": audio_at, "timing": timing})
        cursor += duration
    return scenes


def stamp(seconds: float) -> str:
    ms = round(seconds * 1000)
    return f"{ms // 3_600_000:02d}:{ms // 60_000 % 60:02d}:{ms // 1000 % 60:02d}.{ms % 1000:03d}"


def captions(scenes: list[dict], fps: int) -> str:
    """Legendas de até duas linhas de 42 caracteres, quebradas em pontuação, sem sobreposição."""
    cues: list[list] = []
    for scene in scenes:
        offset = (scene["from"] + scene["audioAt"]) / fps
        words = scene["timing"]["words"]
        tokens = re.findall(r"\S+", scene["voice"])
        if len(tokens) != len(words):
            # As palavras canônicas não carregam pontuação; usa o texto escrito quando alinhado.
            tokens = [word["text"] for word in words]
        chunk: list[tuple[str, dict]] = []
        for token, word in zip(tokens, words):
            chunk.append((token, word))
            text = " ".join(item[0] for item in chunk)
            ends_sentence = token[-1] in ".?!:"
            span = (word["endMs"] - chunk[0][1]["startMs"]) / 1000
            if len(text) >= 70 or ends_sentence or span > 5.5:
                cues.append([offset + chunk[0][1]["startMs"] / 1000, offset + word["endMs"] / 1000, text])
                chunk = []
        if chunk:
            cues.append([offset + chunk[0][1]["startMs"] / 1000, offset + chunk[-1][1]["endMs"] / 1000, " ".join(i[0] for i in chunk)])
    lines = ["WEBVTT", ""]
    for index, (start, end, text) in enumerate(cues):
        # Segura a legenda um instante depois da fala, sem invadir a próxima.
        hold = end + 0.15 if index + 1 == len(cues) else min(end + 0.15, cues[index + 1][0])
        if len(text) > 42:
            words = text.split()
            best = min(range(1, len(words)), key=lambda k: abs(len(" ".join(words[:k])) - len(text) / 2))
            text = " ".join(words[:best]) + "\n" + " ".join(words[best:])
        lines += [f"{stamp(start)} --> {stamp(max(hold, start + 0.2))}", text, ""]
    return "\n".join(lines)


def measure_loudness(source: Path) -> tuple[dict, str]:
    """Primeira passagem do loudnorm: −16 LUFS integrados, pico real −1,5 dBTP."""
    probe = subprocess.run(
        ["ffmpeg", "-hide_banner", "-i", str(source), "-af", "loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json", "-f", "null", "-"],
        capture_output=True, text=True, check=True,
    )
    # O ffmpeg imprime o bloco JSON seguido de mais linhas de status.
    measured, _ = json.JSONDecoder().raw_decode(probe.stderr[probe.stderr.rindex("{"):])
    filter_ = (
        "loudnorm=I=-16:TP=-1.5:LRA=11:"
        f"measured_I={measured['input_i']}:measured_TP={measured['input_tp']}:"
        f"measured_LRA={measured['input_lra']}:measured_thresh={measured['input_thresh']}:"
        f"offset={measured['target_offset']}:linear=true"
    )
    return measured, filter_


def master(source: Path, target: Path, audio_filter: str) -> None:
    """Mestre 1080p: vídeo do Remotion intacto, áudio normalizado."""
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(source), "-c:v", "copy",
         "-af", audio_filter, "-ar", "48000", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", str(target)],
        check=True,
    )


def web(source: Path, target: Path, audio_filter: str) -> None:
    """Versão do site: 1080p H.264 ajustado para gráfico chapado, GOP de 2 s para os capítulos."""
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(source),
         "-c:v", "libx264", "-preset", "slow", "-tune", "animation", "-crf", str(WEB_CRF),
         "-profile:v", "high", "-pix_fmt", "yuv420p", "-g", "60", "-keyint_min", "60",
         "-af", audio_filter, "-ar", "48000", "-c:a", "aac", "-b:a", WEB_AUDIO, "-movflags", "+faststart", str(target)],
        check=True,
    )


def duration_of(path: Path) -> float:
    return float(subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", str(path)],
        capture_output=True, text=True, check=True,
    ).stdout)


def poster(source: Path, at: float, target: Path) -> None:
    subprocess.run(
        ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-ss", f"{at:.2f}", "-i", str(source),
         "-frames:v", "1", "-vf", "scale=1280:720:flags=lanczos", "-c:v", "libwebp", "-quality", "88", str(target)],
        check=True,
    )


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value: dict) -> None:
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")


def publish(
    script_path: Path, script: dict, composition: str, raw: Path, audio_filter: str, out: Path, chapters: list[dict], transcript: str
) -> None:
    """Grava a versão web no site (do render bruto: uma só compressão de áudio) e o manifesto."""
    lesson_id = script.get("lesson")
    if lesson_id:
        name = f"zt-academy-{lesson_id}-v4"
        manifest_path = EDITORIAL / f"academy-{lesson_id}-manifest.json"
        transcript_path = EDITORIAL / f"academy-{lesson_id}-transcript.txt"
        course = json.loads(COURSE_FILE.read_text(encoding="utf-8"))
        training = json.loads(TRAINING_FILE.read_text(encoding="utf-8"))
        lesson = next(item for module in course["modules"] for item in module["lessons"] if item["id"] == lesson_id)
        if lesson_id not in render.MOTION_LESSONS:
            raise SystemExit(f"{lesson_id} não está em render.MOTION_LESSONS")
        content_hash = render.lesson_fingerprint(lesson, training)
    else:
        name = f"zt-{script['id']}-v1"
        manifest_path = EDITORIAL / f"{script['id']}-manifest.json"
        transcript_path = EDITORIAL / f"{script['id']}-transcript.txt"
        content_hash = None
    web(raw, VIDEOS / f"{name}.mp4", audio_filter)
    (VIDEOS / f"{name}.vtt").write_text((out / f"{script['id']}.vtt").read_text(encoding="utf-8"), encoding="utf-8", newline="\n")
    (VIDEOS / f"{name}.webp").write_bytes((out / f"{script['id']}.webp").read_bytes())
    transcript_path.write_text(transcript + "\n", encoding="utf-8", newline="\n")
    target = VIDEOS / f"{name}.mp4"
    assets = render.publication_assets(name)
    narration_hash = render.motion_narration_fingerprint(script)
    manifest = {
        **({"lessonId": lesson_id, "contentHash": content_hash} if lesson_id else {"filmId": script["id"]}),
        "filename": name,
        "title": script["title"],
        "renderer": "remotion",
        "rendererVersion": MOTION_RENDER_VERSION,
        "composition": composition,
        "script": script_path.as_posix(),
        "scriptHash": render.motion_script_fingerprint(script),
        "bettingRuleVersion": script.get("bettingRuleVersion"),
        "durationSeconds": round(duration_of(target), 2),
        "width": 1920,
        "height": 1080,
        "fps": script["fps"],
        "bytes": target.stat().st_size,
        "voice": f"{script['voice']} (synthetic)",
        "voiceRate": script.get("rate", "+0%"),
        "pronunciationVersion": render.MOTION_PRONUNCIATION_VERSION,
        "narrationHash": narration_hash,
        "chapters": chapters,
        "sha256": sha256(target),
        **assets,
    }
    write_json(manifest_path, manifest)
    video = {
        "publicationStatus": "published",
        "url": f"/videos/{name}.mp4",
        "captionsUrl": f"/videos/{name}.vtt",
        "posterUrl": f"/videos/{name}.webp",
        "durationSeconds": manifest["durationSeconds"],
        "transcript": transcript,
        "chapters": chapters,
        "renderer": "remotion",
        "rendererVersion": MOTION_RENDER_VERSION,
        "pronunciationVersion": render.MOTION_PRONUNCIATION_VERSION,
        "narrationHash": narration_hash,
        "mediaRevision": assets["mediaRevision"],
        "bettingRuleVersion": script.get("bettingRuleVersion"),
    }
    if lesson_id:
        # Relê o curso: um texto alterado durante o render cancela a publicação.
        course = json.loads(COURSE_FILE.read_text(encoding="utf-8"))
        lesson = next(item for module in course["modules"] for item in module["lessons"] if item["id"] == lesson_id)
        if render.lesson_fingerprint(lesson, json.loads(TRAINING_FILE.read_text(encoding="utf-8"))) != content_hash:
            raise SystemExit(f"Aula {lesson_id} mudou durante a produção: publicação cancelada")
        lesson["video"] = {**video, "contentHash": content_hash}
        write_json(COURSE_FILE, course)
    else:
        films = json.loads(FILMS_FILE.read_text(encoding="utf-8")) if FILMS_FILE.exists() else {}
        films[script["id"]] = {"filename": name, "title": script["title"], **{k: v for k, v in video.items() if k not in ("url", "captionsUrl", "posterUrl")}}
        write_json(FILMS_FILE, dict(sorted(films.items())))
    print(f"Publicado localmente: {name} · {manifest['durationSeconds']:.2f}s · {manifest['bytes']} bytes")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("composition")
    parser.add_argument("script", type=Path)
    parser.add_argument("--poster", type=float, default=None, help="segundo da capa")
    parser.add_argument("--reuse-render", action="store_true", help="reaproveita out/<id>-raw.mp4")
    parser.add_argument("--publish", action="store_true", help="grava a versão web, o manifesto e o bloco do site")
    args = parser.parse_args()

    script = json.loads((HERE / args.script).read_text(encoding="utf-8"))
    timings = json.loads((HERE / "src/generated" / f"{script['id']}.timings.json").read_text(encoding="utf-8"))
    fps = script["fps"]
    scenes = plan(script, timings)
    out = PILOT / script["id"]
    out.mkdir(parents=True, exist_ok=True)
    raw = HERE / "out" / f"{script['id']}-raw.mp4"
    raw.parent.mkdir(exist_ok=True)

    if not (args.reuse_render and raw.exists()):
        subprocess.run(
            [NODE, str(REMOTION), "render", "src/index.ts", args.composition, str(raw),
             "--codec=h264", "--crf=18", "--x264-preset=slow", "--audio-bitrate=320k",
             "--concurrency=6", "--log=error"],
            cwd=HERE, check=True,
        )
    final = out / f"{script['id']}.mp4"
    measured, audio_filter = measure_loudness(raw)
    master(raw, final, audio_filter)

    (out / f"{script['id']}.vtt").write_text(captions(scenes, fps), encoding="utf-8", newline="\n")
    chapters, seen = [], set()
    for scene in scenes:
        if scene["chapter"] not in seen:
            seen.add(scene["chapter"])
            chapters.append({"start": round(scene["from"] / fps, 2), "title": scene["chapter"]})
    (out / "chapters.json").write_text(json.dumps(chapters, ensure_ascii=False, indent=1), encoding="utf-8")
    transcript = "\n\n".join(scene["voice"] for scene in script["scenes"])
    (out / "transcript.txt").write_text(transcript + "\n", encoding="utf-8", newline="\n")

    poster_at = args.poster if args.poster is not None else scenes[min(1, len(scenes) - 1)]["from"] / fps + 2.5
    poster(final, poster_at, out / f"{script['id']}.webp")
    report = {
        "id": script["id"],
        "durationSeconds": round(duration_of(final), 2),
        "bytes": final.stat().st_size,
        "fps": fps,
        "voice": timings["voice"],
        "engine": timings.get("engine"),
        "loudnessBefore": {"I": measured["input_i"], "TP": measured["input_tp"]},
        "chapters": chapters,
    }
    (out / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False))
    if args.publish:
        publish(args.script, script, args.composition, raw, audio_filter, out, chapters, transcript)


if __name__ == "__main__":
    main()
