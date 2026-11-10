# Security

`course-integrity` 0.1.x is a local command-line program. It does not run a service and it does not send drafts anywhere.

Report a defect through a GitHub issue. Include the version, the command, and a synthetic file that shows the problem. Do not attach student coursework, a corpus database, or secrets.

Privacy of local databases is described in [docs/privacy.md](docs/privacy.md). The practical risk is someone reading the SQLite file you created with `--db`, because that file contains the original text.
