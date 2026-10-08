"""Regression tests for pronunciation, canonical captions and speech cache identity."""
import unittest
from unittest.mock import patch

import render


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


if __name__ == '__main__':
    unittest.main()
