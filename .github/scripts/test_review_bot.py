"""Parser smoke tests for review_bot.py. Run with `python3 -m unittest` (stdlib)."""

import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from review_bot import parse_review_json, _first_balanced_object, finding_fid  # noqa: E402

SAMPLE = {
    "verdict": "changes_requested",
    "summary": "summary here",
    "risk_level": "medium",
    "risk_sources": ["lib.rs:1 - risk"],
    "findings": [
        {"severity": "bug", "title": "Bug", "description": "d",
         "file": "lib.rs", "line": 3, "suggestion": "fix it"},
    ],
}
SAMPLE_JSON = json.dumps(SAMPLE)


class ParseReviewJsonTest(unittest.TestCase):
    def test_direct_json(self):
        self.assertEqual(parse_review_json(SAMPLE_JSON), SAMPLE)

    def test_fenced_json(self):
        self.assertEqual(parse_review_json(f"```json\n{SAMPLE_JSON}\n```"), SAMPLE)

    def test_fenced_no_lang(self):
        self.assertEqual(parse_review_json(f"```\n{SAMPLE_JSON}\n```"), SAMPLE)

    def test_trailing_prose(self):
        wrapped = f"Here is the review:\n{SAMPLE_JSON}\nHope that helps!"
        self.assertEqual(parse_review_json(wrapped), SAMPLE)

    def test_fenced_trailing_prose(self):
        wrapped = f"Here:\n```json\n{SAMPLE_JSON}\n```\nHope that helps!"
        self.assertEqual(parse_review_json(wrapped), SAMPLE)

    def test_invalid_returns_none(self):
        self.assertIsNone(parse_review_json("not json at all"))

    def test_empty(self):
        self.assertIsNone(parse_review_json(""))


class FirstBalancedObjectTest(unittest.TestCase):
    def test_balanced(self):
        self.assertEqual(_first_balanced_object('abc {"a": 1} def'), '{"a": 1}')

    def test_nested(self):
        self.assertEqual(_first_balanced_object('{"a": {"b": [1,2]}} tail'),
                         '{"a": {"b": [1,2]}}')

    def test_string_braces_ignored(self):
        self.assertEqual(_first_balanced_object('{"a": "}"}'), '{"a": "}"}')

    def test_no_object(self):
        self.assertEqual(_first_balanced_object("no braces here"), "")


class FindingFidTest(unittest.TestCase):
    def test_stable_across_runs(self):
        f = {"file": "a.rs", "line": 42, "title": "T"}
        self.assertEqual(finding_fid(f), finding_fid(f))

    def test_distinct(self):
        a = finding_fid({"file": "a.rs", "title": "x"})
        b = finding_fid({"file": "a.rs", "title": "y"})
        self.assertNotEqual(a, b)

    def test_ignores_wobbly_line_number(self):
        # The same finding should keep the same FID even if the line number shifts.
        a = finding_fid({"file": "a.rs", "line": 1, "title": "Unused arg"})
        b = finding_fid({"file": "a.rs", "line": 420, "title": "Unused arg"})
        self.assertEqual(a, b)


if __name__ == "__main__":
    unittest.main()
