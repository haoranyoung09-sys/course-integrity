# course-integrity

Local-first coursework integrity **precheck** for English-medium undergraduate writing, including lab reports and group discussion drafts, with Chinese text supported in the same command.

It reads UTF-8 `.txt` and `.md` files on your machine. It describes sentence length, repetition, vocabulary, and a fixed list of transition phrases. If you point it at a local database you built yourself, it also shows exact and near-exact overlap with those files.

Descriptive review signals only. These figures are not an AI probability, not a calibrated risk score, not a confidence interval, and not an accuracy claim. They are insufficient evidence for authorship. A similarity match is not a plagiarism verdict.

This report has no overall score. The counts are not weighted, summed, or converted into a risk label.

Chinese quickstart: [README.zh-CN.md](README.zh-CN.md).

This is not Turnitin, CNKI, or a replacement for either. It does not connect to them, it does not contain their corpora, and it does not claim to reproduce their results. It does not use the network.

## What you can do with 0.1.0

- `scan` a file or a folder of `.txt` and `.md` drafts.
- Print Markdown, JSON, or both.
- `corpus add` those files into a SQLite database you name with `--db`.
- `corpus check` a draft against only that database.
- See the source label, paragraph number, and a short excerpt for each hit, plus the containment denominator.

DOCX, PDF, and trained models are not in this version. See [docs/roadmap.md](docs/roadmap.md).

## Build

Install Rust 1.83 or newer, then:

```bash
cargo build --locked --release
./target/release/course-integrity --help
```

Building downloads the Rust crates once. Running the tool does not.

The license is [MIT](LICENSE).

## Scan a draft

```bash
cargo run --locked -- scan examples/lab-report-en.md
cargo run --locked -- scan examples/lab-report-zh.md --format json
```

Exit code 0 means the command finished. It does not mean the draft is acceptable or original.

Without `--db`, overlap is not checked. The report says that. An absent match is not evidence of originality, because nothing was compared.

## Read the report

- **Evidence level** is `insufficient`, `limited`, or `descriptive_only`. Those words describe whether the counts are stable enough to talk about. They are not a judgment of the writer.
- **Sentence length** is in tokens. An English word, a number, and one Han character are all one token, so English and Chinese lengths are not on the same scale.
- **Vocabulary** is a type-token ratio with numbers left out. It drops as a text gets longer. Below 80 non-numeric tokens it is marked unstable.
- **Repetition** lists repeated sentences and repeated three-token phrases. Lab methods repeat units. That is described, not scored.
- **Transitions** are a fixed phrase list, counted for description. `first` / `second` / `then` are left out on purpose because lab steps use them constantly.
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
```

`corpus add` stores the **full original text** of each file that has a passage of at least 40 normalized characters, plus a normalized copy and the shingles. Short files are skipped and are not written. See [docs/privacy.md](docs/privacy.md).

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

Everything under `fixtures/synthetic/` is synthetic too. Those files are for tests and demos. They are not student work, and they are not labeled as human-written or AI-written. See [fixtures/README.md](fixtures/README.md).

## Privacy

No network, no telemetry, no upload. The only copy of a draft is the file you already have, plus a database if you pass `--db`, plus a report if you pass `--out-dir` or redirect stdout. Do not commit real coursework. `.gitignore` ignores `*.db`, `.course-integrity/`, `reports/`, `data/`, and `*.review.json` / `*.review.md`.

## Development

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Version

0.1.0. This crate is `publish = false` and is not published to crates.io.
