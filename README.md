# course-integrity

[中文说明](README.zh-CN.md)

[![Declared Rust version 1.83 or newer. CI tests toolchain 1.83.0](https://img.shields.io/badge/Rust-1.83%2B-000000?logo=rust&logoColor=white)](Cargo.toml)
[![CI status for cargo fmt, cargo test, and cargo clippy](https://github.com/haoranyoung09-sys/course-integrity/actions/workflows/ci.yml/badge.svg)](https://github.com/haoranyoung09-sys/course-integrity/actions/workflows/ci.yml)
[![License MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crate version 0.3.0 in this repository. Not published to crates.io and not a GitHub release](https://img.shields.io/badge/version-0.3.0%20source%20only-lightgrey)](Cargo.toml)

Local-first coursework integrity **precheck** for English-medium undergraduate writing, including lab reports and group discussion drafts, with Chinese text supported in the same command.

It reads UTF-8 `.txt` and `.md` on your machine, and it extracts text from `.docx` and from text-based PDFs. It describes sentence length, repetition, vocabulary, and a fixed list of transition phrases. It also lists hidden characters and unusual newlines. If you point it at a local database you built yourself, it shows exact and near-exact overlap with those files. Quotations, bibliography sections, and files you pass with `--boilerplate` are listed separately and are not treated as the main overlap evidence.

Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

This report has no overall score. The counts are not weighted, summed, or converted into a risk label.

Corpus comparison and the character inventory are not AI detection. A red/green token watermark can be checked only with the exact scheme, tokenizer, key, and parameters. This program does not have those parameters and does not guess one.

This is not Turnitin, CNKI, or a replacement for either. It does not connect to them, it does not contain their corpora, and it does not claim to reproduce their results. It does not use the network.

## What 0.3.0 actually does

| Capability | Status |
| --- | --- |
| UTF-8 `.txt` and `.md` | Read directly. The file is not modified |
| `.docx` | Text extracted from `word/document.xml` and header/footer XML, with paragraph locations. Images are counted and skipped |
| Text-based `.pdf` | Text extracted per page. Empty pages are reported. Font encodings can still fail |
| Scanned PDF, photos, charts | Not read. OCR is not included |
| Legacy `.doc` | Not read. Save as `.docx` |
| Hidden characters and newlines | Listed with line, column, and `U+` code point. Not an AI rate |
| Markdown, JSON, and offline HTML | Written locally. HTML has no scripts and no remote assets |
| Local SQLite corpus | Schema version 1, unchanged since 0.1.0. Only extracted plain text is stored for DOCX and PDF |
| Quotations, bibliographies, `--boilerplate` | Shown under "set aside". Still shingle overlap, not a meaning comparison |
| Network, crates.io release, GitHub release | Not used. `publish = false` |

`rust-version` in `Cargo.toml` is 1.83, and CI installs Rust 1.83.0. That is the declared and tested floor. Older compilers were not measured.

## Build

Install Rust 1.83 or newer, then, from a checkout of this repository:

```bash
cargo build --locked --release
./target/release/course-integrity --help
```

The first build downloads the locked crates. Running the tool does not use the network. There is no install step and no crates.io package.

The license is [MIT](LICENSE).

## Demo

From a clean checkout:

```bash
bash scripts/demo.sh
```

The script scans the synthetic English lab note, the Chinese lab note, the English DOCX, and a small hidden-character fixture. It adds `fixtures/synthetic/shared_prompt.md` to a temporary database, checks `fixtures/synthetic/group_draft.md`, and deletes that database. Browse:

- [examples/demo/lab-report-en.html](examples/demo/lab-report-en.html)
- [examples/demo/lab-report-zh.html](examples/demo/lab-report-zh.html)
- [examples/demo/group-draft.html](examples/demo/group-draft.html)
- [examples/demo/hidden-marks.html](examples/demo/hidden-marks.html)

Those HTML files are the output of that script. They are synthetic. See [examples/demo/README.md](examples/demo/README.md).

## Scan a draft

```bash
cargo run --locked -- scan examples/lab-report-en.md
cargo run --locked -- scan examples/lab-report-zh.md --format json
cargo run --locked -- scan fixtures/synthetic/lab-report-en.docx --format html --out-dir /tmp/ci-report
cargo run --locked -- scan fixtures/synthetic/hidden-marks.txt
```

`--format all` writes `.review.md`, `.review.html`, and `.review.json`. It needs `--out-dir`. Each written path is printed on stderr so a JSON report on stdout stays parseable.

Exit code 2 means the command line was wrong, including `--format all` without `--out-dir`. Exit code 1 means a file or database could not be read. The message names the path and the limit when a DOCX zip or a PDF is refused.

Exit code 0 means the command finished. It does not mean the draft is acceptable or original.

Without `--db`, overlap is not checked. The report says that. An absent match is not evidence of originality, because nothing was compared.

## Read the report

- **Evidence level** is `insufficient`, `limited`, or `descriptive_only`. Those words describe whether the counts are stable enough to talk about. They are not a judgment of the writer.
- **Sentence length** is in tokens. An English word, a number, and one Han character are all one token, so English and Chinese lengths are not on the same scale.
- **Vocabulary** is a type-token ratio with numbers left out. It drops as a text gets longer. Below 80 non-numeric tokens it is marked unstable.
- **Repetition** lists repeated sentences and repeated three-token phrases. Lab methods repeat units. That is described, not scored.
- **Transitions** are a fixed phrase list, counted for description. `first` / `second` / `then` are left out on purpose because lab steps use them constantly.
- **Hidden characters** include a byte order mark, CRLF or bare CR in a UTF-8 file, trailing spaces, no-break spaces, zero-width characters, and bidi controls. Each hit has a line, a column, and a code point. Ideographic spaces are ordinary in Chinese and are not listed as hidden controls. DOCX and PDF newline style is not reported, because the extractor inserts paragraph breaks.
- **Corpus hits** use containment. The denominator is the number of distinct 4-token shingles in the query passage. The default threshold is 0.8 (8000 basis points) and at least 4 shared shingles. Jaccard is printed beside it and is not the threshold.

The full rules are in [docs/methodology.md](docs/methodology.md). Every JSON report copies those rules into a `method` object so the file can be read later without this repository.

## Local corpus

Create the database somewhere you do not commit. The path is required. There is no default database, so a scan cannot quietly start storing drafts.

```bash
mkdir -p .course-integrity
cargo run --locked -- corpus add fixtures/synthetic/shared_prompt.md \
  --db .course-integrity/demo.db
cargo run --locked -- corpus check fixtures/synthetic/group_draft.md \
  --db .course-integrity/demo.db
cargo run --locked -- corpus stats --db .course-integrity/demo.db
cargo run --locked -- corpus list --db .course-integrity/demo.db
cargo run --locked -- corpus remove --db .course-integrity/demo.db --id 1
```

`corpus remove --id` deletes that row and its passages from the local file only.

Compare a draft with the lab handout without storing the handout:

```bash
cargo run --locked -- scan fixtures/synthetic/group_draft.md \
  --boilerplate fixtures/synthetic/shared_prompt.md \
  --db .course-integrity/demo.db
```

`corpus add` stores the full original text of each UTF-8 file that has a passage of at least 40 normalized characters, or the extracted plain text of a DOCX or PDF, plus a normalized copy and the shingles. Short files are skipped and are not written. The original DOCX or PDF bytes are not stored. See [docs/privacy.md](docs/privacy.md).

`corpus list` does not print the text. The text is still in the database.

### Self-comparison and duplicates

- Adding the same normalized document again does not insert a second row. The command reports `duplicate` and the existing id.
- Checking a file skips stored rows with the **same source label**, so a draft is not reported as overlapping its own stored copy.
- The same text under a **different label** is a match. A copied file should still show up.
- `--include-self` compares a file with rows stored under its own label, which is useful if you want to see an earlier copy you saved with that same label.

Similarity is still not a plagiarism verdict. The group-draft fixture matches `shared_prompt.md` because the prompt paragraph is shared on purpose.

## Examples

| File | What it is |
| --- | --- |
| [examples/lab-report-en.md](examples/lab-report-en.md) | Synthetic English lab note |
| [examples/lab-report-zh.md](examples/lab-report-zh.md) | Synthetic Chinese lab note |
| [examples/group-discussion.md](examples/group-discussion.md) | Synthetic group discussion that reuses a prompt |
| [fixtures/synthetic/hidden-marks.txt](fixtures/synthetic/hidden-marks.txt) | Synthetic CRLF, no-break space, zero-width space, and a bidi control |

Everything under `fixtures/synthetic/` is synthetic too. Those files are not student work, and they are not labeled as human-written or AI-written. See [fixtures/README.md](fixtures/README.md).

## Privacy

No network, no telemetry, no upload. The scan does not change the file you named. The only extra copy of a draft is a database if you pass `--db`, plus a report if you pass `--out-dir` or redirect stdout. Do not commit real coursework. `.gitignore` ignores `*.db`, `.course-integrity/`, `reports/`, `data/`, and `*.review.json` / `*.review.md` / `*.review.html`.

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Version

0.3.0 in `Cargo.toml`. This crate is `publish = false`. It is not on crates.io, and this repository has no release tag.
