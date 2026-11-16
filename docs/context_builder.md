TODO:

Esim:

Miten Base ja Project yhdistetään?
Mitä ladataan aina?
Mitä haetaan vain tarvittaessa?
Miten vältetään liian suuret promptit?

## Files that are never read (ADR-0012)

Context and search read only Markdown, and skip any file or folder whose name
looks like a secret, even inside granted folders: `*.key`, `*.pem`, `*.p12`,
`*.kdbx`, `id_rsa*`, `id_ed25519*`, `.env*`, and any name containing `secret`,
`password` or `token` (case-insensitive). A folder with such a name is not
entered at all. A session lists what it skipped (the CLI's summary, the
desktop chat's context list), so nothing disappears silently. The rule lives
in `looks_like_secret` in `workspace.rs`.
