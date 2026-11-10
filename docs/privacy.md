# Privacy and security

`course-integrity` is a local program. A scan does not open a network connection, does not send telemetry, and does not call a hosted model.

## What is stored

Nothing is stored unless you run `corpus add` or another command with `--db`.

This local SQLite database stores the full original text of each added file that had a matchable passage, plus a normalized copy of each passage and its shingles. The tool does not upload that text. Do not commit the database, and do not point it at coursework you are not allowed to keep.

Each stored document also has the source label you passed (usually the path), a SHA-256 of the normalized text, the character count, the passage count, and a UTC timestamp. `corpus list` prints the label, hash, counts, and time. It does not print the text. The text is still in the file and anyone who can read that file can read the coursework.

Files with no passage of at least 40 normalized characters are not inserted.

## Where it lives

You choose the path with `--db`. A relative path stays relative. Parent directories are created on this machine when they are missing. The repository `.gitignore` ignores `*.db`, `.course-integrity/`, `reports/`, and `data/`. That does not protect a database you put in a synced folder such as a shared drive or a cloud-sync directory.

Prefer a path outside any folder you commit or share. Delete the database when the course no longer needs it. Deleting the file is the deletion step; the tool has no remote copy to erase.

## What the report contains

JSON and Markdown reports quote matching passages, up to 400 characters each. Treat a report as coursework data. The default command prints it to the terminal. `--out-dir` writes it to files. Those files are ignored when they use the `*.review.json` and `*.review.md` names, but a copy you place elsewhere is your responsibility.

## Security notes

This version has no accounts, no encryption, and no access control. The person who can read the database can read every text that was added. Do not add another student's file unless your course allows you to keep that copy.

The tool only reads the files you name. It does not execute them. It refuses paths that are not UTF-8 `.txt` or `.md`, and it refuses a single file larger than 2 MiB.

There is no network service in this repository. If you find a bug that writes outside the `--db` or `--out-dir` path you passed, open a GitHub issue. Do not attach real student work, a corpus database, or secrets. Version 0.1.x is the supported line for this first release.

The license is MIT. It does not add a rule about who may run the program. Course rules and privacy law still apply to the texts you choose to store.
