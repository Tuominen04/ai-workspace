# Contract: Ollama `/api/chat` (consumed by `RuntimeManager::send_chat_message`)

## Overview

Runtime Manager depends on Ollama's local HTTP chat API. This contract documents the exact request/response shape relied upon, so a future runtime swap (LM Studio, vLLM) has a clear compatibility target and so tests can mock this shape without a live Ollama installation.

## Endpoint

`POST http://127.0.0.1:11434/api/chat`

## Request Body

```json
{
  "model": "qwen3.6:latest",
  "messages": [
    { "role": "system", "content": "..." },
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." },
    { "role": "user", "content": "..." }
  ],
  "stream": false
}
```

- `model`: resolved per Model Selection (project override, else workspace default).
- `messages`: full session history so far, oldest first; the first entry is always the `system` message derived from `ContextSnapshot::as_system_prompt()`.
- `stream`: always `false` for v0.4.1 (see research.md Decision 2).

## Response Body (success, HTTP 200)

```json
{
  "model": "qwen3.6:latest",
  "created_at": "...",
  "message": { "role": "assistant", "content": "..." },
  "done": true
}
```

Only `message.role` and `message.content` are read; other fields are ignored.

## Behavioral Rules

- A non-200 HTTP status maps to `AppError::RuntimeRequestFailed`.
- A 200 response whose body cannot be parsed as the shape above maps to `AppError::RuntimeRequestFailed`.
- The caller (`session.rs`) does not retry automatically; a failure is surfaced to the user, who may ask again.
- `send_chat_message` takes the base URL as a parameter rather than hardcoding it, so tests can point it at a local mock listener instead of a real Ollama instance.
