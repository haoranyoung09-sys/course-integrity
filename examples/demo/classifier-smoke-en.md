# course-integrity 0.4.0 precheck

> Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

There is no overall score in this report.

## examples/lab-report-en.md

- Origin: utf-8
- Encoding of stored text: utf-8
- Bytes: 1134
- Characters: 1134
- BOM stripped: no

### Hidden characters and newlines

Scope: original UTF-8 file. Newline style: lf.

- Hidden-character and newline notes are a deterministic inventory of code points and line breaks in the text that was read. They are not an AI-generation rate, not a plagiarism verdict, and not evidence of who typed the file. A red/green token watermark can be checked only when the exact scheme, tokenizer, key, and parameters are supplied. This tool does not have those parameters and does not guess a watermark.
- Newline style is lf. CRLF and bare CR are listed up to 8 times each; further copies are counted in truncated_count.
- No hidden controls or unusual spaces were found. For a UTF-8 file, unusual newlines would be listed here too.

No listed artifacts.

### Optional local classifier

- Status: ok

- paragraph 1 language en characters 0–138
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 96.063
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 33 of 512
- paragraph 2 language en characters 0–27
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 38.1024
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 10 of 512
- paragraph 3 language en characters 0–101
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 99.466
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 25 of 512
- paragraph 4 language en characters 0–9
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 53.8459
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 4 of 512
- paragraph 5 language en characters 0–258
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 72.3675
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 61 of 512
- paragraph 6 language en characters 0–7
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 42.8105
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 4 of 512
- paragraph 7 language en characters 0–161
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 9.2844
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 59 of 512
- paragraph 8 language en characters 0–10
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 56.6981
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 4 of 512
- paragraph 9 language en characters 0–406
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 5.4137
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 85 of 512

Classifier-flagged character share: 0.462 (516/1117 scored characters).

Classifier-flagged character share uses only characters inside chunks that were actually scored. A chunk is flagged when its AI-class softmax is greater than 0.5. The denominator is that scored-character count, not the length of the file. This share is not the portion of the file that is AI-written.

Coverage: 1117 scored characters, 0 skipped characters, 0 characters not covered by a tokenizer window.

### What was read

- Read as UTF-8 text. Paragraph locations are blank-line blocks in the file.
- DOCX text is read from word/document.xml and from word header and footer XML. Paragraph locations name that file and the paragraph number. PDF text is read from each page's text layer, and locations name the page. Images, drawings, charts, text boxes, and legacy .doc files are not read. A Word or PDF file is text extraction, not multimodal reasoning.

### Evidence limits

- Level: `descriptive_only`
- Authorship: Insufficient evidence for authorship. These signals do not identify a person or a writing process.
- The text is long enough for the variation and vocabulary figures to be marked stable. That still does not identify an author or a writing process.
- Insufficient evidence for authorship. These signals do not identify a person or a writing process.

### Notes

- Unvalidated surface heuristics are reported as descriptive review signals. They are not an AI probability, a calibrated risk score, a confidence interval, or an accuracy claim.
- This report has no overall score. The counts are not weighted, summed, or converted into a risk label.
- Similarity is not a plagiarism verdict. Shared prompts, quotations, templates, group work, and ordinary technical phrasing can produce a match.
- Technical lab prose often uses short procedural sentences, repeated units, and number-heavy lines. Those patterns are described here and are not treated as an integrity finding.
- The median sentence is very short. Procedural lab lines are often short, and this tool does not score that pattern.
- No local corpus was queried because --db was not set. Overlap was not checked. This is not a plagiarism check and it does not contact a remote service.

### Document statistics

- Paragraphs: 9
- Sentences: 26
- Tokens: 203 words, 0 Han or kana, 18 numeric

#### Sentence length

Sentence length counts tokens: an English word, a numeric run, or one Han or kana character. English word lengths and Chinese character lengths are not on the same scale, and this tool does not compare them. Sample standard deviation uses the n-1 divisor and is omitted below 2 sentences. Variation is marked stable only at 5 or more sentences. Reported decimals are rounded to 4 places.

| Measure | Value |
| --- | --- |
| unit | tokens |
| count | 26 |
| min | 1.0000 |
| max | 20.0000 |
| mean | 8.5000 |
| median | 7.0000 |
| sample standard deviation | 5.4571 |
| coefficient of variation | 0.6420 |
| stable | yes |

Document sentence length uses 26 values, which meets the reporting minimum of 5.

#### Repetition

Repeated phrases are trigrams of the non-numeric token stream that occur at least 3 times. Numeric tokens are left out of that stream. Trigrams whose three tokens are all common English function words are omitted. At most 8 phrases are listed. Duplicate sentences are exact matches after the same normalization used for corpus identity, and only when the normalized sentence has at least 20 characters. Repetition is a description, not a verdict.

- Duplicate sentence types: 0
- Extra copies: 0
- Examples: none

- `at s it` × 6
- `c at s` × 6
- `it was c` × 6
- `s it was` × 6
- `was c at` × 6
- `we did not` × 3

#### Vocabulary

- Non-numeric tokens: 203
- Unique: 107
- Type-token ratio: 0.5271
- Hapax tokens: 82
- Hapax ratio: 0.4039
- Numeric tokens excluded: 18
- Stable: yes

The vocabulary ratio uses 203 non-numeric tokens, which meets the reporting minimum of 80. Type-token ratio is unique non-numeric tokens divided by all non-numeric tokens in the counted span. It falls as texts get longer. Chinese tokens are characters, so a Chinese ratio is not comparable to an English word ratio. Below 80 non-numeric tokens the ratio is marked unstable. It is not an authorship score.

#### Transition inventory

Transition phrases are a fixed inventory counted for description. English phrases match consecutive word tokens, longest phrase first, and a numeric or Han token breaks the word run. Chinese phrases match the Han and kana character stream with separators removed, longest phrase first, without overlapping. Step words such as first, second, and then are not in the inventory, because lab procedures repeat them constantly. A high or low count is not a style verdict and is not evidence about authorship.

- Total hits: 2
- Per 100 sentences: 7.6923

- `however`: 1
- `therefore`: 1

### Paragraphs

#### Paragraph 1 (paragraph 1)

> Synthetic teaching example for course-integrity. Not a student submission. Not labeled as human-written or AI-written. Not ground truth.

- Sentences: 4
- Tokens: 21 words, 0 Han or kana, 0 numeric
- Mean sentence length: 5.2500
- Type-token ratio: 0.8571
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 2 (paragraph 2)

# Cooling a beaker of water

- Sentences: 1
- Tokens: 5 words, 0 Han or kana, 0 numeric
- Mean sentence length: 5.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 3 (paragraph 3)

Course context: a fictional first-year lab, PHYS 199. The numbers below were chosen for this example.

- Sentences: 2
- Tokens: 16 words, 0 Han or kana, 1 numeric
- Mean sentence length: 8.5000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 4 (paragraph 4)

## Method

- Sentences: 1
- Tokens: 1 words, 0 Han or kana, 0 numeric
- Mean sentence length: 1.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 5 (paragraph 5)

We poured 200 ml of tap water into a glass beaker and heated it until the water reached 60 C. The beaker then stood on the open bench. We recorded the temperatu…

- Sentences: 4
- Tokens: 48 words, 0 Han or kana, 3 numeric
- Mean sentence length: 12.7500
- Type-token ratio: 0.6875
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 6 (paragraph 6)

## Data

- Sentences: 1
- Tokens: 1 words, 0 Han or kana, 0 numeric
- Mean sentence length: 1.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 7 (paragraph 7)

At 0 s the temperature was 60 C. At 30 s it was 57 C. At 60 s it was 55 C. At 90 s it was 53 C. At 120 s it was 51 C. At 150 s it was 50 C. At 180 s it was 49 C…

- Sentences: 7
- Tokens: 36 words, 0 Han or kana, 14 numeric
- Mean sentence length: 7.1429
- Type-token ratio: 0.1944
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 8 (paragraph 8)

## Comment

- Sentences: 1
- Tokens: 1 words, 0 Han or kana, 0 numeric
- Mean sentence length: 1.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 9 (paragraph 9)

The temperature fell quickly at first and then more slowly. However, we never measured the room temperature, so we cannot say how close the water came to the ro…

- Sentences: 5
- Tokens: 74 words, 0 Han or kana, 0 numeric
- Mean sentence length: 14.8000
- Type-token ratio: 0.7162
- Transition hits: 2
- Extra duplicate sentences: 0

### Local corpus

No local corpus was queried because --db was not set. Overlap was not checked. This is not a plagiarism check and it does not contact a remote service.

### Method

- UTF-8 text is NFKC-normalized and lowercased with Unicode simple lowercase (not full casefold). A word token is a run of letters, including letters with diacritics, and an apostrophe stays inside a word when letters surround it. A numeric token is a run of ASCII digits with an optional internal decimal point or comma between digits. Each Han character, kana character, or CJK compatibility ideograph is its own token. Punctuation, symbols, emoji, and a leading byte-order mark are separators, not tokens. Digits split mixed tokens, so H2O becomes three tokens. Hyphens are separators. There is no Chinese dictionary word segmenter.
- Three counts are used. Sentence length counts every token, including numbers. Vocabulary and type-token ratio omit numeric tokens and still include function words. Shingles keep every token, including numbers and function words, so a copied measurement sentence can still match. English function words are removed only when a whole repeated trigram is made of them.
- A shingle is an ordered sequence of 4 consecutive tokens from one paragraph, joined by single spaces. Comparison uses the set of distinct shingles, so a repeated shingle counts once. Punctuation is not a token, so a shingle can cross a comma or a line break inside the paragraph. A passage with fewer than 4 distinct shingles, or shorter than 40 normalized characters, is not near-exact compared. The same width is used for English words and for Chinese characters.
- Containment = |distinct query shingles intersect distinct source shingles| / |distinct query shingles|. The denominator is the number of distinct 4-token shingles in the query passage, not the source passage, not the character length, and not a probability. The ratio is rounded half-up to 4 decimal places. containment_basis_points uses the same rounding, and 10000 means 1. The threshold comparison uses those basis points.
- Jaccard = |intersection| / |union| of those same distinct shingle sets. It is reported beside containment, with the same half-up rounding and jaccard_basis_points. The near-exact threshold uses containment only. The default threshold is 0.8 (8000 basis points), together with at least 4 shared shingles.
- Files must be valid UTF-8. A leading U+FEFF is ignored for statistics and does not change the normalized hash. CR, LF, and CRLF are line breaks. A blank line (Unicode whitespace only) separates paragraphs. Compatibility forms are folded with NFKC before tokenization and hashing, so fullwidth Latin letters match ASCII letters. The degree Celsius sign folds under NFKC; the degree mark is a separator. Case folding is Unicode simple lowercase, so German eszett is not expanded to ss.
- An exact passage match means the normalized passage strings are equal and the normalized form has at least 40 characters. Normalization for identity is NFKC, simple lowercase, punctuation and symbols removed, and whitespace collapsed to single spaces. Shorter passages are not emitted as matches, because short technical phrases collide often.
- Document identity for storage deduplication is the SHA-256 hex digest of the normalized full text, not of the raw bytes. The database still keeps the original text, including a leading BOM if one was present. Adding the same normalized document again does not insert a second copy.
- Checking a file excludes stored rows with the same source label, so a file is not reported as overlapping its own stored copy. Identical normalized text under a different label is reported as a match. Pass --include-self to compare a file with rows stored under the same label. Self-exclusion is by label, not by hash: a classmate file with the same text and a different label is still a match.
- Sentence length counts tokens: an English word, a numeric run, or one Han or kana character. English word lengths and Chinese character lengths are not on the same scale, and this tool does not compare them. Sample standard deviation uses the n-1 divisor and is omitted below 2 sentences. Variation is marked stable only at 5 or more sentences. Reported decimals are rounded to 4 places.
- Type-token ratio is unique non-numeric tokens divided by all non-numeric tokens in the counted span. It falls as texts get longer. Chinese tokens are characters, so a Chinese ratio is not comparable to an English word ratio. Below 80 non-numeric tokens the ratio is marked unstable. It is not an authorship score.
- Repeated phrases are trigrams of the non-numeric token stream that occur at least 3 times. Numeric tokens are left out of that stream. Trigrams whose three tokens are all common English function words are omitted. At most 8 phrases are listed. Duplicate sentences are exact matches after the same normalization used for corpus identity, and only when the normalized sentence has at least 20 characters. Repetition is a description, not a verdict.
- Transition phrases are a fixed inventory counted for description. English phrases match consecutive word tokens, longest phrase first, and a numeric or Han token breaks the word run. Chinese phrases match the Han and kana character stream with separators removed, longest phrase first, without overlapping. Step words such as first, second, and then are not in the inventory, because lab procedures repeat them constantly. A high or low count is not a style verdict and is not evidence about authorship.
- This report has no overall score. The counts are not weighted, summed, or converted into a risk label.
- DOCX text is read from word/document.xml and from word header and footer XML. Paragraph locations name that file and the paragraph number. PDF text is read from each page's text layer, and locations name the page. Images, drawings, charts, text boxes, and legacy .doc files are not read. A Word or PDF file is text extraction, not multimodal reasoning.
- Image and scan OCR is not included. If a PDF page or a DOCX has no extractable text, the report says so and stops. Figures and charts are not interpreted. No external OCR program is called.
- A passage is set aside when it is mostly a quotation, when it follows a references or bibliography heading, or when it matches a file passed with --boilerplate. Set-aside overlap is still 4-token shingle overlap. It is not a semantic reading, and it is not listed as the main overlap evidence.
- Database schema version stays 1 in this release. A 0.1.0 database opens without migration. DOCX and PDF bytes are not stored; only the extracted plain text is stored, and only when a passage is long enough to compare.
- Hidden-character and newline notes are a deterministic inventory of code points and line breaks in the text that was read. They are not an AI-generation rate, not a plagiarism verdict, and not evidence of who typed the file. A red/green token watermark can be checked only when the exact scheme, tokenizer, key, and parameters are supplied. This tool does not have those parameters and does not guess a watermark.

### Limitations

- DOCX and text-based PDF are extracted to plain text. Legacy .doc, scanned pages, images, and charts are not read. OCR is not included.
- Writing statistics are not a trained-model score. The optional classifier, when it is run, reports an uncalibrated class-1 softmax. That number is not a probability that the text was written by AI.
- Chinese text is tokenized per character. There is no dictionary word segmenter.
- Near-exact matching uses 4-token shingles. Paraphrase, summary, and translation are not detected.
- Sentence boundaries are heuristic. A small set of abbreviations and decimal points is handled; other dots can split or join sentences.
- Short text makes length, variation, and vocabulary figures unstable. The report says so when that is the case.
- Technical lab prose and shared assignment wording can look repetitive or similar without being misconduct.
- A match shows overlap with a document you added locally. It does not show who copied whom, or that a rule was broken.
- Digits split mixed tokens, so a formula such as H2O becomes separate tokens. Chemistry writeups can look less diverse for that reason alone.

