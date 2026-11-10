# Method

`course-integrity` 0.1.0 reports descriptive review signals for UTF-8 `.txt` and `.md` drafts. This page is the contract for those signals. Schema version of the local database is 1. A later change to tokenization or shingles will refuse an older database file instead of quietly rescoring it.

The sentences below are the same strings the report prints.

Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

This report has no overall score. The counts are not weighted, summed, or converted into a risk label.

## Tokenizer

UTF-8 text is NFKC-normalized and lowercased with Unicode simple lowercase (not full casefold). A word token is a run of letters, including letters with diacritics, and an apostrophe stays inside a word when letters surround it. A numeric token is a run of ASCII digits with an optional internal decimal point or comma between digits. Each Han character, kana character, or CJK compatibility ideograph is its own token. Punctuation, symbols, emoji, and a leading byte-order mark are separators, not tokens. Digits split mixed tokens, so H2O becomes three tokens. Hyphens are separators. There is no Chinese dictionary word segmenter.

## What each count includes

Three counts are used. Sentence length counts every token, including numbers. Vocabulary and type-token ratio omit numeric tokens and still include function words. Shingles keep every token, including numbers and function words, so a copied measurement sentence can still match. English function words are removed only when a whole repeated trigram is made of them.

Sentence length counts tokens: an English word, a numeric run, or one Han or kana character. English word lengths and Chinese character lengths are not on the same scale, and this tool does not compare them. Sample standard deviation uses the n-1 divisor and is omitted below 2 sentences. Variation is marked stable only at 5 or more sentences. Reported decimals are rounded to 4 places.

Type-token ratio is unique non-numeric tokens divided by all non-numeric tokens in the counted span. It falls as texts get longer. Chinese tokens are characters, so a Chinese ratio is not comparable to an English word ratio. Below 80 non-numeric tokens the ratio is marked unstable. It is not an authorship score.

Repeated phrases are trigrams of the non-numeric token stream that occur at least 3 times. Numeric tokens are left out of that stream. Trigrams whose three tokens are all common English function words are omitted. At most 8 phrases are listed. Duplicate sentences are exact matches after the same normalization used for corpus identity, and only when the normalized sentence has at least 20 characters. Repetition is a description, not a verdict.

Transition phrases are a fixed inventory counted for description. English phrases match consecutive word tokens, longest phrase first, and a numeric or Han token breaks the word run. Chinese phrases match the Han and kana character stream with separators removed, longest phrase first, without overlapping. Step words such as first, second, and then are not in the inventory, because lab procedures repeat them constantly. A high or low count is not a style verdict and is not evidence about authorship.

Paragraphs are split on blank lines. Sentence boundaries are `. ! ? ; 。 ！ ？ ； …`, plus newlines inside a paragraph. Decimal points between digits are kept, and a small abbreviation list (`Dr.`, `Fig.`, `e.g.`, `i.e.`, `et al.`, and a few others) does not split. Other dots can still split or join a sentence. That is a limitation, not a hidden score.

Evidence levels are labels about stability, not about authorship:

- `insufficient` — empty, or fewer than 2 sentences, or fewer than 20 non-numeric tokens, or fewer than 80 characters.
- `limited` — more than that, but still under 5 sentences, or under 80 non-numeric tokens, or under 400 characters.
- `descriptive_only` — at or above those floors. The figures can be described. They still do not identify an author.

Short technical lines and number-heavy lab tables are described in the notes. They are not turned into a finding.

## Shingles, containment, and the denominator

A shingle is an ordered sequence of 4 consecutive tokens from one paragraph, joined by single spaces. Comparison uses the set of distinct shingles, so a repeated shingle counts once. Punctuation is not a token, so a shingle can cross a comma or a line break inside the paragraph. A passage with fewer than 4 distinct shingles, or shorter than 40 normalized characters, is not near-exact compared. The same width is used for English words and for Chinese characters.

Containment = |distinct query shingles intersect distinct source shingles| / |distinct query shingles|. The denominator is the number of distinct 4-token shingles in the query passage, not the source passage, not the character length, and not a probability. The ratio is rounded half-up to 4 decimal places. containment_basis_points uses the same rounding, and 10000 means 1. The threshold comparison uses those basis points.

Jaccard = |intersection| / |union| of those same distinct shingle sets. It is reported beside containment, with the same half-up rounding and jaccard_basis_points. The near-exact threshold uses containment only. The default threshold is 0.8 (8000 basis points), together with at least 4 shared shingles.

Worked example, locked by the unit test `worked_containment_example`:

- Query tokens: `alpha bravo charlie delta echo foxtrot golf hotel` (8 tokens, 5 distinct shingles).
- Source tokens: `alpha bravo charlie delta echo foxtrot golf` (7 tokens, 4 distinct shingles).
- Shared shingles: 4. Those are all of the source shingles.
- Containment = 4 / 5 = 0.8000, which is 8000 basis points. The denominator is 5.
- Jaccard = 4 / 5 = 0.8000, which is 8000 basis points.

A light edit in the middle of a short passage can miss 0.8 because each changed token touches up to 4 shingles. This version does not detect paraphrase, summary, or translation. A miss is not evidence that the draft is original.

## Unicode

Files must be valid UTF-8. A leading U+FEFF is ignored for statistics and does not change the normalized hash. CR, LF, and CRLF are line breaks. A blank line (Unicode whitespace only) separates paragraphs. Compatibility forms are folded with NFKC before tokenization and hashing, so fullwidth Latin letters match ASCII letters. The degree Celsius sign folds under NFKC; the degree mark is a separator. Case folding is Unicode simple lowercase, so German eszett is not expanded to ss.

## Exact match, storage hash, and excerpts

An exact passage match means the normalized passage strings are equal and the normalized form has at least 40 characters. Normalization for identity is NFKC, simple lowercase, punctuation and symbols removed, and whitespace collapsed to single spaces. Shorter passages are not emitted as matches, because short technical phrases collide often.

Document identity for storage deduplication is the SHA-256 hex digest of the normalized full text, not of the raw bytes. The database still keeps the original text, including a leading BOM if one was present. Adding the same normalized document again does not insert a second copy.

Excerpts are the original passage text with whitespace collapsed, truncated to 400 characters. Paragraph indexes are 1-based and count blank-line paragraphs, including short ones that are not themselves matchable.

A file is stored only when at least one passage reaches 40 normalized characters. Shorter files are skipped and their text is not written into the database. Passages inside a stored file that are themselves short are kept, because the file had something matchable, but those short passages are not emitted as hits.

## Self-comparison and deduplication

Checking a file excludes stored rows with the same source label, so a file is not reported as overlapping its own stored copy. Identical normalized text under a different label is reported as a match. Pass --include-self to compare a file with rows stored under the same label. Self-exclusion is by label, not by hash: a classmate file with the same text and a different label is still a match.

Labels are the path you passed, with backslashes rewritten to slashes and a leading `./` removed. The path is not rewritten to an absolute path. `--label` overrides that for one file. It is rejected for a directory, so a folder of drafts does not collapse onto one label.

Adding the same normalized text twice does not create a second row. The second add reports `duplicate` and the existing id. Checking a different label still matches that one stored copy.

## Coverage

Coverage is only this local database. It is not the web, not an institution-wide archive, and not a commercial checker corpus. This tool does not connect to Turnitin, CNKI, or any other remote service, and it does not claim to reproduce their results.

This local SQLite database stores the full original text of each added UTF-8 file that had a matchable passage, or the extracted plain text of a DOCX or PDF, plus a normalized copy of each passage and its shingles. The original DOCX or PDF bytes are not stored. The tool does not upload that text. Do not commit the database, and do not point it at coursework you are not allowed to keep.

An empty corpus reports zero documents. No overlap can be found. That is not evidence that a draft is original. `scan` without `--db` does not look for overlap at all.

## DOCX, PDF, and where a passage came from

DOCX text is read from word/document.xml and from word header and footer XML. Paragraph locations name that file and the paragraph number. PDF text is read from each page's text layer, and locations name the page. Images, drawings, charts, text boxes, and legacy .doc files are not read. A Word or PDF file is text extraction, not multimodal reasoning.

Image and scan OCR is not included. If a PDF page or a DOCX has no extractable text, the report says so and stops. Figures and charts are not interpreted. No external OCR program is called.

Database schema version stays 1 in this release. A 0.1.0 database opens without migration. DOCX and PDF bytes are not stored; only the extracted plain text is stored, and only when a passage is long enough to compare.

DOCX limits, in addition to the 8 MiB file cap: at most 128 zip entries, 8 MiB uncompressed per entry, and 32 MiB uncompressed in total. A DOCTYPE or ENTITY declaration in the Word XML is refused. Zip entry names that contain `..` or are absolute are skipped.

## Quotations, bibliographies, and handouts

A passage is set aside when it is mostly a quotation, when it follows a references or bibliography heading, or when it matches a file passed with --boilerplate. Set-aside overlap is still 4-token shingle overlap. It is not a semantic reading, and it is not listed as the main overlap evidence.

`--boilerplate` compares in memory. Those files are not written to the database unless you also `corpus add` them. A whole-document exact hash match is still reported, because an entire copied file is a different signal from one shared reference entry.

HTML reports are a single file with no scripts and no remote assets. Text from the draft is escaped. Opening the HTML file does not upload it.

## Limits that are easy to forget

- Chinese is characters, not dictionary words. Two essays can share character shingles because the writing is ordinary, not because a passage was copied.
- Group work, a shared prompt, a methods template, and a quotation can match exactly. The report says so on every hit. It does not decide misconduct.
- There is no single score to "lower". The tool does not advise how to change a draft so that a checker returns a smaller number.
