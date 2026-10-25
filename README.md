# R3Viewer

A terminal viewer for local files and Nushell data, with an interactive reader and inline output.

Read text and source code, inspect JSON, YAML, TOML and XML, or explore tables, lists and nested records from a Nushell pipeline. Syntax highlighting, search and keyboard navigation help you inspect content directly in your terminal.

## Install

Install from source on **Windows** with **Nushell 0.116.1**. Prebuilt downloads are not available. Linux and macOS are not supported installation targets yet.

You need [Git](https://git-scm.com/downloads), [Rust via rustup](https://rustup.rs/), the Windows C++ build tools, and [Nushell 0.116.1](https://github.com/nushell/nushell/releases/tag/0.116.1). The repository selects Rust 1.99.0 automatically through rustup.

In PowerShell, clone and build:

```powershell
git clone https://github.com/R3Neer/R3Viewer.git
cd R3Viewer
cargo build --locked --release --workspace --bins
New-Item -ItemType Directory -Path bin -Force | Out-Null
Copy-Item target/release/r3viewer.exe, target/release/nu_plugin_r3viewer.exe bin/
```

Keep the `bin/` and `nu/` folders together. The module locates its executable relative to itself, so you can use `viewer` from any working directory.

Open Nushell in the cloned repository and run these commands **one at a time**:

```nu
plugin add ("bin/nu_plugin_r3viewer.exe" | path expand)
plugin use r3viewer
use nu/r3viewer.nu viewer
"Hello from Nushell" | viewer -i
```

To make `viewer` available in new sessions, add these lines to your Nushell `config.nu`, replacing the module path with its absolute location:

```nu
plugin use r3viewer
use 'D:/Tools/R3Viewer/nu/r3viewer.nu' viewer
```

## Use

Open a local file in the interactive reader:

```nu
viewer notes.txt
viewer src/main.rs
viewer settings.json --structure
```

Inspect values directly from a pipeline:

```nu
ls | select name type size | viewer
{name: 'Ana', scores: [8 9 10]} | viewer
open data.csv | where active == true | viewer
```

An argument names a file. Pipeline input supplies its value: `"notes.txt" | viewer` displays the literal text `notes.txt`. Use `--path/-p` to interpret a string as a local path, or `--value/-v` to treat an argument as literal content. These flags cannot be combined.

```nu
"notes.txt" | viewer --path
viewer --value 'Hello from the terminal'
```

Print into the terminal scrollback with `--inline/-i`:

```nu
viewer notes.txt -i
{name: 'Ana', score: 42} | viewer -i
viewer notes.txt -i | save rendered.txt
```

Redirected output is plain text. `viewer` consumes its input and produces a presentation; apply data transformations before passing values to it.

The standalone executable can also read files or text from standard input:

```powershell
./bin/r3viewer.exe notes.txt
'Hello from PowerShell' | ./bin/r3viewer.exe -i
```

## Keyboard controls

| Key | Action |
| --- | --- |
| Arrow keys or `h`, `j`, `k`, `l` | Navigate or scroll |
| Page Up / Page Down | Move by a page |
| Home / End | Move to the beginning / end |
| Enter | Open the selected value |
| `r` / `K` | Inspect the selected row / key |
| Tab / Shift+Tab | Switch between source and parsed structure |
| `o` | Inspect the selected node's original source |
| Backspace | Return to the parent value |
| `/` | Search for literal, case-sensitive text |
| `n` / `N` | Next / previous match |
| `L` | Toggle line numbers |
| `?` | Show help |
| `q` | Quit |
| Escape | Cancel search or quit |
| Ctrl+C | Cancel and exit |

## Display options

| Option | Purpose |
| --- | --- |
| `--format auto\|text\|code\|json\|yaml\|toml\|xml\|markdown` | Choose how to interpret input |
| `--language rust` | Select a syntax grammar by name or extension |
| `--structure` | Start in the parsed structure of a data file |
| `--line-numbers/-n` | Show line numbers |
| `--no-line-numbers` | Hide line numbers |
| `--colour auto\|always\|never` | Select colour behaviour; redirection stays plain |
| `--theme auto\|dark\|light` | Select a display palette |
| `--config path/to/viewer.toml` | Load a TOML configuration file |

R3Viewer is a read-only viewer. Local data files open as source by default. Nushell pipeline values retain their types, including nested containers, binary data and null. Appearance follows the terminal and supported static Nushell colour settings; no separate R3CLI installation is required.

Markdown has a formatted text presentation with `viewer notes.md -i` and in redirected output. Its interactive view currently shows source text. Images, Mermaid, graphical mathematics, PDF viewing and web URLs are not available in the user-facing command yet.

Run `viewer --help` in Nushell or `r3viewer --help` for command help.

## Troubleshooting

- If `viewer` is unknown, load the plugin and module in the order shown under Install.
- If the module cannot find its executable, check that `bin/` and `nu/` are siblings and that both executables were copied into `bin/`.
- To use the standalone executable by name, add the absolute `bin/` directory to your PATH.

## Licence

[MIT](LICENSE). Third-party components retain their own licences and notices in `vendor/`.
