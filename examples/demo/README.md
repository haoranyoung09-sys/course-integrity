# Synthetic demo output

These files are produced by `scripts/demo.sh` from the synthetic examples in this repository. They are not student work. The script builds the local binary, scans the fixtures, writes reports, and deletes the temporary corpus database. Nothing is uploaded.

Open `lab-report-en.html` or `lab-report-zh.html` in a browser. `group-draft.html` is a corpus check against the shared prompt. `hidden-marks.html` lists CRLF, a no-break space, a zero-width space, and a bidi control as code points.

`lab-report-en.png`, `hidden-marks.png`, and `group-draft.png` are headless Chrome screenshots of those HTML files, captured from the top of the page.

Regenerate with:

```bash
bash scripts/demo.sh
```
