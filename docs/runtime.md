TODO:

Esim:

Miten Ollama käynnistetään?
Miten tarkistetaan, onko se jo käynnissä?
Miten vaihdetaan myöhemmin toiseen runtimeen?

## Starting Ollama (ADR-0016)

`runtime` in `Config/workspace.toml` says how Ollama is started when nothing answers on
`127.0.0.1:11434`; whatever already answers there is always used instead.

- `"ollama"` (the default): `ollama serve`, stopped again when the session ends.
- `"podman"`: the Ollama image as the container `ai-workspace-ollama`, with
  `podman run --detach --rm --pull=never --publish 127.0.0.1:11434:11434
  --volume ai-workspace-ollama:/root/.ollama <image>`, plus `--device nvidia.com/gpu=all`
  for `gpu = "nvidia"` or `--device /dev/kfd --device /dev/dri` (and the `rocm` image) for
  `gpu = "amd"`. A missing image is reported with its `podman pull` command; the app never
  pulls. A container the app started is stopped (and so removed) when the session ends; a
  leftover stopped container of that name is removed first; any other container is never
  touched. The volume keeps the models.

## A model's own instructions (ADR-0012)

When a session opens, the runtime asks Ollama about the session's model
(`POST /api/show`). If the model's Modelfile has a `SYSTEM` prompt, every
request's system message is that prompt first, unchanged, followed by the
workspace context. Ollama uses a request's system message instead of the
Modelfile's, so without this the model's rules would be dropped. A model
without a `SYSTEM` prompt gets the context alone, as before. If Ollama cannot
be asked, the session goes on with the context alone, and both front ends say
so (the CLI's `Model:` line, the desktop chat's model line).

## Streamed replies (ADR-0013)

Chat requests use `stream: true`: Ollama sends the reply as newline-delimited
JSON pieces, and the app reads them as they come. The request waits up to 10
seconds to connect and up to 5 minutes for each next piece (room for a large
model to load before its first word), with no limit on the whole answer. A
piece carrying `error`, or a stream that ends without `done`, fails the
request. The desktop app shows the reply as it grows; the command line prints
it when it is complete. Everything still goes only to `127.0.0.1:11434`.
