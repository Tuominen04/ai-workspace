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

## How much fits (ADR-0012)

A chat session sizes its context from the model's context window (`num_ctx`,
reported by Ollama): workspace text gets 1.5 bytes per window token, minus the
model's own instructions, and up to one source per 1,500 bytes (between 8 and
32). An 8,192-token model gets about 12 kB, a 32,768-token model about 48 kB.
When the window is unknown, the fixed 12,000 bytes and 8 sources apply.

Sources are taken in relevance order while they fit. The first one that does
not fit is cut at a line break, if at least 512 bytes of it fit, and the text
says so: `[truncated: N of M bytes shown]`. Smaller files later in the order can
still fit whole. Sessions show the budget and any cut file (the CLI summary,
the desktop chat's context list).

## What is read (ADR-0014)

`context::readable_sources` defines it once, for both chat context and search:

- Markdown in all of `Base/` and in the whole selected project folder
  (`Knowledge/`, `Specs/`, `Resources/`, `Session/`, and the project's top
  level), three levels deep;
- the session's active temporary grants: a granted `.md` file, or the `.md`
  files in a granted folder, two levels deep, shown as "Granted".

The block list and the budget apply to all of it. A grant counts from the
next message and stops counting when it is revoked or the session ends.
