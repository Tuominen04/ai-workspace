TODO:

Esim:

Miten Ollama käynnistetään?
Miten tarkistetaan, onko se jo käynnissä?
Miten vaihdetaan myöhemmin toiseen runtimeen?

## A model's own instructions (ADR-0012)

When a session opens, the runtime asks Ollama about the session's model
(`POST /api/show`). If the model's Modelfile has a `SYSTEM` prompt, every
request's system message is that prompt first, unchanged, followed by the
workspace context. Ollama uses a request's system message instead of the
Modelfile's, so without this the model's rules would be dropped. A model
without a `SYSTEM` prompt gets the context alone, as before. If Ollama cannot
be asked, the session goes on with the context alone, and both front ends say
so (the CLI's `Model:` line, the desktop chat's model line).
