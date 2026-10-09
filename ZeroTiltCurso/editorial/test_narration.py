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
        import pineapple
        self.assertEqual(pineapple.cue_time({'words': words}, 'check-raise', render.speech_text), .25 + 200 / 1e7)

    def test_incomplete_or_different_speech_is_rejected(self):
        with self.assertRaises(ValueError):
            render.canonical_boundaries('Pineapple e river', self.boundaries('painépou'))
        with self.assertRaises(ValueError):
            render.canonical_boundaries('Pineapple', self.boundaries('Pineapple'))

    def test_invalid_timing_is_rejected(self):
        with self.assertRaises(ValueError):
            render.canonical_boundaries('Call', [{'text': 'cól', 'offset': -1, 'duration': 100}])

    def test_changed_guide_or_voice_invalidates_narration(self):
        scenes = [{'voice': 'Pineapple.'}]
        before = render.narration_fingerprint(scenes)
        with patch.dict(render.PRONUNCIATIONS, {'pineapple': 'outra pronúncia'}):
            self.assertNotEqual(before, render.narration_fingerprint(scenes))
        with patch.object(render, 'VOICE_RATE', '+0%'):
            self.assertNotEqual(before, render.narration_fingerprint(scenes))
        self.assertEqual(scenes, [{'voice': 'Pineapple.'}])


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
        scenes=json.loads((render.HERE/'episodes.json').read_text(encoding='utf-8'))['pineapple']['scenes']
        for scene in scenes:
            if not scene.get('actions'): continue
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


if __name__ == '__main__':
    unittest.main()
