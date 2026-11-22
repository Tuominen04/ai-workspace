# Contribution Rules

Found a bug or have an idea? Open an issue with the **Bug report** or **Feature request** form.
Questions go to [Discussions](https://github.com/Tuominen04/ai-workspace/discussions), and
security problems to a private report (see `SECURITY.md`). Pull requests are welcome; every one
is reviewed and merged by the maintainer, and the checks must pass first.

- All new features start with a spec or clear written intent.
- Documentation comes before implementation.
- Keep pull requests focused on one meaningful feature.
- Do not introduce changes that break the existing architecture.
- If architecture changes, update `ARCHITECTURE.md` and record the decision in `DECISIONS.md`.

## Testing Guidelines

- Add tests when a change affects behavior, workflows, validation, or safety.
- Cover at least one happy path and the critical failure or safety paths.
- Do not add tests for every tiny refactor, rename, formatting-only edit, or wording change.
- Prefer a small set of high-signal tests over many low-value tests.
