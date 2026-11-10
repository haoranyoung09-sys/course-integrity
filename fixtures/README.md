# Synthetic fixtures

Every file under `synthetic/` is invented for tests and demos. None of it is a student submission. None of it is ground truth for "human" or "AI" authorship. Do not add real coursework here.

`empty.txt` and `short.txt` are intentionally blank or tiny, so tests can see those cases. The other files start with `Synthetic fixture.` That line is too short to be stored or reported as a corpus match.

`shared_prompt.md` and `group_draft.md` share one paragraph on purpose. A hit between them shows shared assignment wording. It is not a plagiarism verdict.

`en_source.md` and `zh_source.md` are the local-corpus sources. The `exact` files repeat one paragraph and then add a different paragraph. The `near` file appends one short sentence to the English source paragraph. The `unrelated` files are about a titration, not the cooling beaker.

`lab-report-en.docx`, `lab-report-zh.docx`, `group-discussion.docx`, and `lab-report-en.pdf` are synthetic documents generated for this repository. The English and Chinese Word files each contain one tiny payload under `word/media` so a scan can show that pictures are skipped. The PDF has a single text-layer sentence, not a scan. None of these files is ground truth for human or AI writing.
