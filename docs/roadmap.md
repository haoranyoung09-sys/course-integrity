# Roadmap

Version 0.1.0 is a local precheck for UTF-8 `.txt` and `.md` drafts. Later items are optional. They are not in this version, and the report does not pretend that they ran.

## Possible later work

- Reading DOCX and PDF by extracting their text, with the same descriptive report and the same statement when extraction is lossy.
- An optional Chinese word segmenter, off by default, so character shingles remain the documented behavior until someone chooses otherwise.
- An index for larger local corpora. The score would stay containment over distinct query shingles.
- Clearer export for a seminar discussion, still without an overall score.

## Not planned

- A trained model that emits a probability of AI use, a risk band, or an accuracy claim.
- A connection to Turnitin, CNKI, or any other commercial or campus corpus, or a claim that local scores match those products.
- Uploads, telemetry, or a paid API.
- Features whose purpose is to lower a similarity number, evade a detector, or guarantee that a draft will pass a checker.

If a change alters tokenization, shingle width, or the score denominator, the database schema version changes and old files are refused until they are rebuilt with `corpus add`.
