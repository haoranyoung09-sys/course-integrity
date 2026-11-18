"""Sidecar tests that do not download models or import torch."""

import json
import subprocess
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from windows import content_windows


class FakeTokenizer:
    def encode(self, text, add_special_tokens=True):
        words = text.split()
        extra = 2 if add_special_tokens else 0
        return [0] * (len(words) + extra)

    def __call__(self, text, add_special_tokens=False, return_offsets_mapping=False, truncation=False, return_attention_mask=False):
        offsets = []
        cursor = 0
        for word in text.split():
            start = text.find(word, cursor)
            end = start + len(word)
            offsets.append((start, end))
            cursor = end
        return {"offset_mapping": offsets}


class WindowTests(unittest.TestCase):
    def test_short_text_is_one_window(self):
        plan = content_windows(FakeTokenizer(), "alpha beta gamma")
        self.assertIsNone(plan["reason"])
        self.assertEqual(len(plan["windows"]), 1)
        self.assertEqual(plan["windows"][0]["char_start"], 0)
        self.assertTrue(plan["windows"][0]["within_limit"])

    def test_long_text_is_split_without_overlap(self):
        words = [f"w{i}" for i in range(600)]
        text = " ".join(words)
        plan = content_windows(FakeTokenizer(), text)
        self.assertGreater(len(plan["windows"]), 1)
        previous_end = -1
        for window in plan["windows"]:
            self.assertGreaterEqual(window["char_start"], previous_end)
            self.assertLessEqual(window["token_count"], 512)
            previous_end = window["char_end"]

    def test_empty_text_is_not_scored_as_zero(self):
        plan = content_windows(FakeTokenizer(), "")
        self.assertEqual(plan["reason"], "no_tokens")
        self.assertEqual(plan["windows"], [])


class MissingModelTests(unittest.TestCase):
    def test_infer_reports_not_run_without_a_model(self):
        script = Path(__file__).resolve().parent / "infer.py"
        completed = subprocess.run(
            [sys.executable, str(script)],
            input=json.dumps(
                {
                    "models_dir": "/tmp/course-integrity-no-such-models",
                    "chunks": [{"id": "p1", "text": "alpha beta", "language": "en"}],
                }
            ),
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        payload = json.loads(completed.stdout)
        self.assertEqual(payload["status"], "not_run")
        self.assertEqual(payload["reason"], "missing_model")
        self.assertNotIn("ai_softmax", json.dumps(payload["chunks"]))


if __name__ == "__main__":
    unittest.main()
