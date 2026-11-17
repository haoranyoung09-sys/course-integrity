#!/usr/bin/env bash
# Reproducible synthetic demo. Does not upload files and does not keep a corpus database.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
DEST="$ROOT/examples/demo"
WORK="$(mktemp -d)"
cleanup() {
  rm -rf "$WORK"
}
trap cleanup EXIT

mkdir -p "$DEST"

run() {
  cargo run --locked --quiet -- "$@"
}

run scan examples/lab-report-en.md --format all --out-dir "$WORK/en" >/dev/null
run scan examples/lab-report-zh.md --format html --out-dir "$WORK/zh" >/dev/null
run scan fixtures/synthetic/lab-report-en.docx --format html --out-dir "$WORK/docx" >/dev/null
run scan fixtures/synthetic/hidden-marks.txt --format html --out-dir "$WORK/marks" >/dev/null
run corpus add fixtures/synthetic/shared_prompt.md --db "$WORK/demo.db" >/dev/null
run corpus check fixtures/synthetic/group_draft.md --db "$WORK/demo.db" --format all --out-dir "$WORK/group" >/dev/null

cp "$WORK/en/lab-report-en.review.html" "$DEST/lab-report-en.html"
cp "$WORK/en/lab-report-en.review.md" "$DEST/lab-report-en.md"
cp "$WORK/zh/lab-report-zh.review.html" "$DEST/lab-report-zh.html"
cp "$WORK/docx/lab-report-en.review.html" "$DEST/lab-report-en-docx.html"
cp "$WORK/marks/hidden-marks.review.html" "$DEST/hidden-marks.html"
cp "$WORK/group/group_draft.review.html" "$DEST/group-draft.html"
cp "$WORK/group/group_draft.review.json" "$DEST/group-draft.json"
cp "$WORK/docx/lab-report-en.review.html" "$ROOT/examples/sample-report.html"

echo "Wrote synthetic demo reports in examples/demo/"
echo "The temporary corpus database was deleted and was not committed."
