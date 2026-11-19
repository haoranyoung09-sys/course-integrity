# course-integrity 0.4.0 precheck

> Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

There is no overall score in this report.

## examples/lab-report-zh.md

- Origin: utf-8
- Encoding of stored text: utf-8
- Bytes: 1030
- Characters: 418
- BOM stripped: no

### Hidden characters and newlines

Scope: original UTF-8 file. Newline style: lf.

- Hidden-character and newline notes are a deterministic inventory of code points and line breaks in the text that was read. They are not an AI-generation rate, not a plagiarism verdict, and not evidence of who typed the file. A red/green token watermark can be checked only when the exact scheme, tokenizer, key, and parameters are supplied. This tool does not have those parameters and does not guess a watermark.
- Newline style is lf. CRLF and bare CR are listed up to 8 times each; further copies are counted in truncated_count.
- No hidden controls or unusual spaces were found. For a UTF-8 file, unusual newlines would be listed here too.

No listed artifacts.

### Optional local classifier

- Status: ok

- paragraph 1 language mixed characters 0–61
  - Not scored: mixed_or_unsupported_language
- paragraph 2 language zh characters 0–6
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 30.2897
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 7 of 512
- paragraph 3 language zh characters 0–29
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 37.6919
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 31 of 512
- paragraph 4 language zh characters 0–5
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 72.2747
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 6 of 512
- paragraph 5 language zh characters 0–82
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 15.9945
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 74 of 512
- paragraph 6 language zh characters 0–5
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 25.7306
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 6 of 512
- paragraph 7 language zh characters 0–102
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 70.1804
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 67 of 512
- paragraph 8 language zh characters 0–5
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 36.4657
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: human
  - Tokens including special tokens: 6 of 512
- paragraph 9 language zh characters 0–106
  - AI 模型分类分数（未校准） / AI classifier score (uncalibrated): 98.6571
  - This number is the softmax of class index 1, scaled from 0 to 100. Class index 0 is human and class index 1 is AI in the official AIGC v3 demo and training code. It is not the percentage of this text that was written by AI, and it is not a calibrated probability of authorship.
  - Predicted class: ai
  - Tokens including special tokens: 108 of 512

Classifier-flagged character share: 0.6265 (213/340 scored characters).

Classifier-flagged character share uses only characters inside chunks that were actually scored. A chunk is flagged when its AI-class softmax is greater than 0.5. The denominator is that scored-character count, not the length of the file. This share is not the portion of the file that is AI-written.

Coverage: 340 scored characters, 61 skipped characters, 0 characters not covered by a tokenizer window.

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
- No local corpus was queried because --db was not set. Overlap was not checked. This is not a plagiarism check and it does not contact a remote service.

### Document statistics

- Paragraphs: 9
- Sentences: 22
- Tokens: 2 words, 277 Han or kana, 17 numeric

#### Sentence length

Sentence length counts tokens: an English word, a numeric run, or one Han or kana character. English word lengths and Chinese character lengths are not on the same scale, and this tool does not compare them. Sample standard deviation uses the n-1 divisor and is omitted below 2 sentences. Variation is marked stable only at 5 or more sentences. Reported decimals are rounded to 4 places.

| Measure | Value |
| --- | --- |
| unit | tokens |
| count | 22 |
| min | 2.0000 |
| max | 39.0000 |
| mean | 13.4545 |
| median | 10.5000 |
| sample standard deviation | 10.0935 |
| coefficient of variation | 0.7502 |
| stable | yes |

Document sentence length uses 22 values, which meets the reporting minimum of 5.

#### Repetition

Repeated phrases are trigrams of the non-numeric token stream that occur at least 3 times. Numeric tokens are left out of that stream. Trigrams whose three tokens are all common English function words are omitted. At most 8 phrases are listed. Duplicate sentences are exact matches after the same normalization used for corpus identity, and only when the normalized sentence has at least 20 characters. Repetition is a description, not a verdict.

- Duplicate sentence types: 0
- Extra copies: 0
- Examples: none

- `摄 氏 度` × 8
- `为 摄 氏` × 7
- `度 秒 时` × 6
- `时 为 摄` × 6
- `氏 度 秒` × 6
- `秒 时 为` × 6
- `们 没 有` × 3
- `我 们 没` × 3

#### Vocabulary

- Non-numeric tokens: 279
- Unique: 153
- Type-token ratio: 0.5484
- Hapax tokens: 105
- Hapax ratio: 0.3763
- Numeric tokens excluded: 17
- Stable: yes

The vocabulary ratio uses 279 non-numeric tokens, which meets the reporting minimum of 80. Type-token ratio is unique non-numeric tokens divided by all non-numeric tokens in the counted span. It falls as texts get longer. Chinese tokens are characters, so a Chinese ratio is not comparable to an English word ratio. Below 80 non-numeric tokens the ratio is marked unstable. It is not an authorship score.

#### Transition inventory

Transition phrases are a fixed inventory counted for description. English phrases match consecutive word tokens, longest phrase first, and a numeric or Han token breaks the word run. Chinese phrases match the Han and kana character stream with separators removed, longest phrase first, without overlapping. Step words such as first, second, and then are not in the inventory, because lab procedures repeat them constantly. A high or low count is not a style verdict and is not evidence about authorship.

- Total hits: 3
- Per 100 sentences: 13.6364

- `然而`: 1
- `因此`: 1
- `所以`: 1

### Paragraphs

#### Paragraph 1 (paragraph 1)

> 这是 course-integrity 的合成教学示例，不是学生作业，也不标注为人写或机器写，不能当作区分二者的标准。

- Sentences: 1
- Tokens: 2 words, 37 Han or kana, 0 numeric
- Mean sentence length: 39.0000
- Type-token ratio: 0.7949
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 2 (paragraph 2)

# 烧杯冷却

- Sentences: 1
- Tokens: 0 words, 4 Han or kana, 0 numeric
- Mean sentence length: 4.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 3 (paragraph 3)

课程背景是虚构的一年级实验。下面的数字是为这个示例选定的。

- Sentences: 2
- Tokens: 0 words, 27 Han or kana, 0 numeric
- Mean sentence length: 13.5000
- Type-token ratio: 0.8889
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 4 (paragraph 4)

## 步骤

- Sentences: 1
- Tokens: 0 words, 2 Han or kana, 0 numeric
- Mean sentence length: 2.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 5 (paragraph 5)

我们向玻璃烧杯中加入 200 毫升自来水，加热到 60 摄氏度，然后把烧杯放在实验台上。之后每隔 30 秒记录一次温度，共记录三分钟。我们没有搅拌，也没有盖上烧杯。

- Sentences: 3
- Tokens: 0 words, 62 Han or kana, 3 numeric
- Mean sentence length: 21.6667
- Type-token ratio: 0.7742
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 6 (paragraph 6)

## 数据

- Sentences: 1
- Tokens: 0 words, 2 Han or kana, 0 numeric
- Mean sentence length: 2.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 7 (paragraph 7)

0 秒时温度为 60 摄氏度。30 秒时为 57 摄氏度。60 秒时为 55 摄氏度。90 秒时为 53 摄氏度。120 秒时为 51 摄氏度。150 秒时为 50 摄氏度。180 秒时为 49 摄氏度。

- Sentences: 7
- Tokens: 0 words, 44 Han or kana, 14 numeric
- Mean sentence length: 8.2857
- Type-token ratio: 0.1591
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 8 (paragraph 8)

## 说明

- Sentences: 1
- Tokens: 0 words, 2 Han or kana, 0 numeric
- Mean sentence length: 2.0000
- Type-token ratio: 1.0000
- Transition hits: 0
- Extra duplicate sentences: 0

#### Paragraph 9 (paragraph 9)

温度一开始下降较快，随后变慢。然而我们没有测量室温，因此不能说明水温离室温还有多远。所以这段记录只描述表格。我们没有拟合冷却模型，也不从七个读数里声称某个物理定律。如果另一组重复实验并补上室温，表格会更容易讨论。

- Sentences: 5
- Tokens: 0 words, 97 Han or kana, 0 numeric
- Mean sentence length: 19.4000
- Type-token ratio: 0.8351
- Transition hits: 3
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

