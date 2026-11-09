# Examples

These files are synthetic teaching examples for an English-medium undergraduate lab and a group discussion. They are not student submissions. They are not labeled as human-written or AI-written, and they are not ground truth for either.

- `lab-report-en.md` — a short cooling-beaker note in English.
- `lab-report-zh.md` — the same kind of note in Chinese.
- `group-discussion.md` — three speakers answering a shared prompt. The prompt is repeated on purpose so a local corpus check can show shared wording without calling it plagiarism.

Try:

```bash
cargo run --locked -- scan examples/lab-report-en.md
cargo run --locked -- scan examples/lab-report-zh.md
```

Corpus commands belong in [the README](../README.md). Use a `--db` path you do not commit.
