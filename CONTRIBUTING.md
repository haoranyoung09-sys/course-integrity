# Contributing

Thanks for helping with `course-integrity`. The project is a small local precheck for undergraduate drafts. Keep it honest and small.

## Development

Rust 1.83 or newer, with rustfmt and clippy:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
```

Those are the same commands CI runs. `Cargo.lock` is committed. Do not add a dependency for something the standard library or the current crates already cover. Do not add a network client.

## What belongs here

- Bug fixes that keep the documented tokenizer, denominator, and Unicode behavior, or that change those rules in `docs/methodology.md` and the report text together.
- Tests for empty input, short input, non-ASCII text, English and Chinese matching, the database round trip, and CLI output.
- Docs that say what the tool does not know.

## What does not belong here

- A score whose purpose is to look like an AI probability, a calibrated risk, or a confidence interval.
- Advice or features for lowering a similarity score or getting past a detector.
- Claims that the tool matches Turnitin, CNKI, or any other product, or that it can see their corpora.
- Extra restrictions inside the MIT license. Course conduct is a course matter. The license stays MIT.
- Real coursework, local databases, generated reports, or secrets. `.gitignore` is there so those stay off the branch. Synthetic fixtures must say they are synthetic and must not be labeled as human or AI ground truth.

## Pull requests

Describe the behavior change and how you tested it. If you change a sentence that the report prints from `src/method.rs`, update `docs/methodology.md` in the same change. The unit test checks that those sentences still appear there.

## Conduct

Be respectful. Read `CODE_OF_CONDUCT.md`.
