# AI Workspace

A local AI workspace for Fedora. You keep your notes in project folders; AI Workspace
hands them to a model running on your own computer, so you can ask about your own work.
Nothing leaves your machine: the model runs in [Ollama](https://ollama.com), the files stay
in a folder you choose, and the app connects to nothing but Ollama on `127.0.0.1`.

## How it works

- **Projects.** A workspace is a folder (`~/AI` unless you choose another) with a `Base`
  folder of notes every project shares, and a `Projects` folder with one folder per project.
- **Chat.** Open a project and chat. The model reads the project's Markdown notes and the
  Base notes, and nothing else unless you grant it more, for that chat only.
- **Ollama.** Each project can use its own Ollama model, for example a general model for one
  project and your own security advisor for another. At the end of a chat you can save a short
  note, which the next chat reads.

Your files are never changed behind your back: the app only writes when you ask it to.

## Quick start

You need Fedora on x86_64.

### 1. Install Ollama and a model

Install Ollama from its [Linux download page](https://ollama.com/download/linux) (a one-line
installer), or follow its [manual install](https://docs.ollama.com/linux#manual-install) steps
if you prefer not to run a script. Then download a model from Ollama [library](https://ollama.com/library?sort=newest), and run:

```bash
ollama pull <model>
```

Any model works; Ollama's [model library](https://ollama.com/library) lists them. Larger models
answer better but need more memory. To use a graphics card, see Ollama's
[GPU page](https://docs.ollama.com/gpu); for anything else, its
[documentation](https://docs.ollama.com).

### 2. Install AI Workspace

1. Download `ai-workspace-<version>.x86_64.rpm` from the latest
   [release](https://github.com/Tuominen04/ai-workspace/releases).
2. Install it:

   ```bash
   sudo dnf install ./ai-workspace-<version>.x86_64.rpm
   ```

3. Open **AI Workspace** from the applications menu, or run `ai-workspace`.

### 3. First chat

1. **New project** in the sidebar: give it a name, for example "Home Lab".
2. Put some Markdown notes (`.md` files) in its folder, `~/AI/Projects/home-lab/`.
3. On the project's **Overview**, **Change** the model to the one you downloaded.
4. Open **Chat**, **Start session**, and ask about your notes.

AI Workspace starts Ollama for you if it is not running.

## Learn more

- [User guide](docs/user-guide.md): every feature, where things are kept, and what to do when
  something goes wrong.
- [Roadmap](ROADMAP.md): what is coming.
- [Vision and design](docs/vision.md) and [development](docs/development.md): for working on
  the project itself.

## Install

The steps above are all there is. To check a download first, see
[verifying a download](docs/release.md#verifying-a-download). Removing the app
(`sudo dnf remove ai-workspace`) leaves your workspace folder and notes untouched.

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
