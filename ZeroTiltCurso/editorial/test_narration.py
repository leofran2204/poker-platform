"""Regression tests for pronunciation, canonical captions and speech cache identity."""
import unittest
from unittest.mock import patch

import render
import timeline
import json


class PronunciationTests(unittest.TestCase):
    def boundaries(self, text):
        return [{'text': token, 'offset': i * 200, 'duration': 100}
                for i, token in enumerate(text.split())]

    def test_requested_pronunciation_without_substring_replacement(self):
        spoken, _ = render.pronunciation_plan('Pineapple, PINEAPPLE! fold; folder; blindado.')
        self.assertEqual(spoken, 'painépou, painépou! fôuld; folder; blindado.')

    def test_many_spoken_words_retain_one_written_term_and_full_timing(self):
        text = 'All-in no Pineapple.'
        spoken, _ = render.pronunciation_plan(text)
        words = render.canonical_boundaries(text, self.boundaries(spoken))
        self.assertEqual([w['text'] for w in words], ['All-in', 'no', 'Pineapple'])
        self.assertEqual(words[0]['offset'], 0)
        self.assertEqual(words[0]['duration'], 300)
        self.assertEqual(words[2]['offset'], 600)
        self.assertTrue(render.complete_speech(text, words))

    def test_apostrophes_compounds_decimals_and_cue_times(self):
        text = 'Hold’em: check-raise para 7,50.'
        spoken, _ = render.pronunciation_plan(text)
        words = render.canonical_boundaries(text, self.boundaries(spoken))
        self.assertEqual([w['text'] for w in words], ['Hold’em', 'check-raise', 'para', '7,50'])
        self.assertEqual(render.cue_time({'words': words}, 'check-raise', render.speech_text), .25 + 200 / 1e7)

    def test_incomplete_or_different_speech_is_rejected(self):
        with self.assertRaises(ValueError):
            render.canonical_boundaries('Pineapple e river', self.boundaries('painépou'))
        with self.assertRaises(ValueError):
            render.canonical_boundaries('Pineapple', self.boundaries('Pineapple'))

    def test_invalid_timing_is_rejected(self):
        with self.assertRaises(ValueError):
            render.canonical_boundaries('Call', [{'text': 'cól', 'offset': -1, 'duration': 100}])

    def test_changed_guide_or_voice_invalidates_narration(self):
        # "poker" continua lido em português ("pôquer"); termos em inglês usam a grafia original.
        scenes = [{'voice': 'Poker e Pineapple.'}]
        before = render.narration_fingerprint(scenes)
        with patch.dict(render.PRONUNCIATIONS, {'poker': 'outra pronúncia'}):
            self.assertNotEqual(before, render.narration_fingerprint(scenes))
        with patch.object(render, 'VOICE_RATE', '+0%'):
            self.assertNotEqual(before, render.narration_fingerprint(scenes))
        with patch.object(render, 'VOICE', 'outra-voz'):
            self.assertNotEqual(before, render.narration_fingerprint(scenes))
        self.assertEqual(scenes, [{'voice': 'Poker e Pineapple.'}])


class TimelineTests(unittest.TestCase):
    def test_every_street_reveals_then_waits_before_private_cards(self):
        for players in (2,3,6):
            for folded in ((),(1,)):
                events=timeline.deal_timeline(.5,2,5,players,folded)
                self.assertEqual(len({(e['kind'],e.get('player'),e['index']) for e in events}),len(events))
                for stage in (3,4,5):
                    board=[e for e in events if e['kind']=='board' and (e['index']<3 if stage==3 else e['index']==stage-1)]
                    holes=[e for e in events if e['kind']=='hole' and e['index']==stage-1]
                    self.assertEqual(len(holes),players-len(folded))
                    self.assertEqual({e['player'] for e in holes},set(range(players))-set(folded))
                    self.assertGreaterEqual(min(e['start'] for e in holes),max(e['end'] for e in board)+2)
                flop=[e for e in events if e['kind']=='board'][:3]
                self.assertAlmostEqual(flop[1]['start']-flop[0]['start'],.35)
                for e in events:
                    self.assertIsNone(timeline.card_state(e,e['start']-.001))
                    self.assertEqual(timeline.card_state(e,e['end']+100)['progress'],1)
                    if e['kind']=='board':
                        self.assertAlmostEqual(e['land']-e['start'],.6)
                        self.assertAlmostEqual(e['flip']-e['land'],1)
                        self.assertAlmostEqual(e['end']-e['flip'],.8)
                        self.assertFalse(timeline.card_state(e,e['flip']-.001)['face'])
                        self.assertTrue(timeline.card_state(e,e['end'])['face'])

    def test_betting_animations_conserve_chips_at_every_frame(self):
        # Blinds 0,50/1, ante de 1 BB do big blind e abertura para 3,50 com dois calls.
        scenes=[{'table':{'pot':0,'players':[{'name':'Ana','stack':10000,'bet':0},
                                             {'name':'Beto','stack':10000,'bet':0},
                                             {'name':'Caio','stack':10000,'bet':0}]},
                 'actions':[{'player':1,'to':50},{'player':2,'to':100},{'player':2,'ante':100},
                            {'player':0,'to':350},{'player':1,'to':350},{'player':2,'to':350}]}]
        for scene in scenes:
            initial=scene['table']; expected=initial['pot']+sum(p['stack'] for p in initial['players'])
            actions=[{**a,'at':i*2} for i,a in enumerate(scene['actions'])]
            for f in range(150):
                state=timeline.ledger(initial,actions,f/15)
                self.assertEqual(state['pot']+sum(p['stack'] for p in state['players']),expected)
            for a in actions:
                if 'ante' in a:
                    before=timeline.ledger(initial,actions,a['at'])['players'][a['player']]['bet']
                    after=timeline.ledger(initial,actions,a['at']+1)['players'][a['player']]['bet']
                    self.assertEqual(before,after)

    def test_animation_only_change_invalidates_publication_revision(self):
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as directory, patch.object(render,'OUT',Path(directory)):
            for ext in ('mp4','vtt','webp'): (Path(directory)/f'film.{ext}').write_bytes(ext.encode())
            old=render.publication_assets('film')
            (Path(directory)/'film.mp4').write_bytes(b'new frames, same voice')
            self.assertNotEqual(old['mediaRevision'],render.publication_assets('film')['mediaRevision'])


class NarrationStandardTests(unittest.TestCase):
    """Padrão de narração de todos os vídeos da plataforma (dono, 19/11/2026)."""

    def test_portuguese_and_english_are_both_marked(self):
        body = render.speech_ssml('Bem-vindo à aula de hoje sobre o Brazilian Pineapple. Antes, o small blind paga.')
        self.assertEqual(body, '<lang xml:lang="pt-BR">Bem-vindo à aula de hoje sobre o</lang> '
                               '<lang xml:lang="en-US">Brazilian Pineapple</lang>. '
                               '<lang xml:lang="pt-BR">Antes, o</lang> <lang xml:lang="en-US">small blind</lang> '
                               '<lang xml:lang="pt-BR">paga.</lang>')

    def test_short_english_heavy_sentences_are_flagged(self):
        self.assertEqual(render.heavy_english_sentences('Float. Um float precisa de um plano claro. Custa 2.5 big blinds no total.'),
                         ['Float.'])

    def test_bb_reads_as_unit_or_seat(self):
        self.assertEqual(render.speak('Vilão no BB paga 1 BB e depois 3 BB.'), 'Vilão no big blind paga 1 big blind e depois 3 big blinds.')

    def test_every_published_narration_follows_the_standard(self):
        from pathlib import Path
        here = Path(render.__file__).resolve().parent
        course = json.loads(render.COURSE_FILE.read_text(encoding='utf-8'))
        training = json.loads(render.TRAINING_FILE.read_text(encoding='utf-8'))
        voices = {}
        for module in course['modules']:
            for lesson in module['lessons']:
                if module['id'] != 'm0' and lesson['id'] not in render.MOTION_LESSONS:
                    voices[lesson['id']] = [s['voice'] for s in render.academy_scenes(lesson, training)]
        episodes = json.loads((here / 'episodes.json').read_text(encoding='utf-8'))
        for key in ('ep11', 'ep12', 'ep13'):
            voices[key] = [s['voice'] for s in episodes[key]['scenes']]
        for name in ('pineapple-draw-rules', 'pineapple-draw-trailer', 'academy-m5l3', 'academy-m5l4', 'academy-m5l5'):
            script = json.loads((here / 'motion/src/data' / f'{name}.json').read_text(encoding='utf-8'))
            self.assertEqual(script['voice'], render.NARRATION_VOICE, name)
            voices[name] = [s['voice'] for s in script['scenes']]
        self.assertEqual(len(voices), 30)
        for key, texts in voices.items():
            self.assertEqual([s for t in texts for s in render.heavy_english_sentences(t)], [], key)


if __name__ == '__main__':
    unittest.main()
