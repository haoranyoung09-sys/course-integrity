# Roadmap

Version 0.2.0 reads UTF-8 `.txt` and `.md`, extracts text from `.docx` and text-based `.pdf`, and writes an offline HTML report as well as Markdown and JSON. The local database is still schema 1.

## Possible later work

- Optional local OCR for scans, behind an explicit flag and a documented external program. It is not in 0.2.0.
- An optional Chinese word segmenter, off by default, so character shingles remain the documented behavior until someone chooses otherwise.
- An index for larger local corpora. The score would stay containment over distinct query shingles.

## In 0.2.0, and still not a verdict

- DOCX and PDF input is text extraction. It does not describe figures or charts.
- Set-aside rows for quotations, bibliographies, and `--boilerplate` files are still shingle matches. They are not a semantic comparison.

## Not planned

- A trained model that emits a probability of AI use, a risk band, or an accuracy claim.
- A connection to Turnitin, CNKI, or any other commercial or campus corpus, or a claim that local scores match those products.
- Uploads, telemetry, or a paid API.
- Features whose purpose is to lower a similarity number, evade a detector, or guarantee that a draft will pass a checker.

If a change alters tokenization, shingle width, or the score denominator, the database schema version changes and old files are refused until they are rebuilt with `corpus add`.
