# S03: catálogo integrado y selección de sintaxis

2026-10-10. Contraste funcional Windows de catálogo, selector, ayuda y versión. [Aceptación independiente](../reviews/S03-catalogo-final.md): SIN_CRITICAS_PENDIENTES en ese alcance. S03, render de gramáticas, QA visual y runtime Unix conservan sus demás validaciones.

## Reglas vigentes

El mismo build genera el catálogo y el pack ordenado de Syntect5.3.0 con78 entradas. Selección: extensión primero (orden inverso del pack), después nombre, sin distinguir mayúsculas; `--language` admite punto inicial. En los dos tokens ambiguos observados, ASP selecciona HTML (ASP) y TeX selecciona LaTeX por esa prioridad. Es la regla existente del selector y worker, conservada.

Auto infiere sólo la extensión final de un archivo real: fixture.css.erb se resuelve por erb; `--language css.erb` usa el token compuesto explícito. Un literal/pipeline sin selector conserva su clasificación, y no se cargan gramáticas de rutas recibidas. El alias pwsh/PWSH explícito selecciona PowerShell; auto .pwsh permanece plano. Los parsers de estructura mantienen selección independiente de `--language`.

Ayuda y versión del binario integrado funcionan exit0 (r3viewer0.1.0); las opciones y los formatos publicados coinciden con el CLI actual. Backend Windows Onig, build fancy-regex; Nushell/PowerShell y TOML usan los recursos locales derivados e identificados en sus manifests. No se actualizan dependencias por este control.

## Evidencia y errores del observador

Observador R3:78 entradas y244 tokens de extensión, todos cotejados con el pack real; dos colisiones de nombre registradas. Manifest antes de outputs4AB08F1997F4794A56DD4DAAA095C5C0D712CAEAE452CD807D75421B7C485F8B; raíz comprobó22/22 entradas idénticas después. R1 imponía prioridad de nombre (falló ASP); R2 confundía suffix compuesto con Path::extension (falló css.erb). Se conservaron ambos fallos y se corrigieron prospectivamente los oráculos a la política original, sin modificar producto para aprobarlos.

Agente independiente:22/22 entradas cotejadas, observador propio enlazado con una copia física de la biblioteca,322 tokens y negativos adicionales; replay R3, help y version con exit0 y hashes estables. Manifest final5814BFA576144E9D0BCD9EE9F4E89293E5D567223A2DCB2A54834786AF34E225 conserva27 artefactos, cotejados27/27 por raíz. Bibliotecas originales no recompiladas por el revisor. Directorios privados: .tmp/s03-catalog-2026-10-10 y .tmp/s03-catalog-adversary-2026-10-10.

## Entradas del pack observado

Esta lista documenta nombres y tokens integrados. La selección respeta la precedencia anterior; la aceptación de representación/coste y corpus de cada gramática permanece separada.

| Sintaxis | Tokens de extensión |
|---|---|
| Plain Text | `txt` |
| ASP | `asa` |
| HTML (ASP) | `asp` |
| ActionScript | `as` |
| AppleScript | `applescript`, `script editor` |
| Batch File | `bat`, `cmd` |
| NAnt Build File | `build` |
| C# | `cs`, `csx` |
| C++ | `cpp`, `cc`, `cp`, `cxx`, `c++`, `C`, `h`, `hh`, `hpp`, `hxx`, `h++`, `inl`, `ipp` |
| C | `c`, `h` |
| CSS | `css`, `css.erb`, `css.liquid` |
| Clojure | `clj` |
| D | `d`, `di` |
| Diff | `diff`, `patch` |
| Erlang | `erl`, `hrl`, `Emakefile`, `emakefile` |
| HTML (Erlang) | `yaws` |
| Go | `go` |
| Graphviz (DOT) | `dot`, `DOT`, `gv` |
| Groovy | `groovy`, `gvy`, `gradle` |
| HTML | `html`, `htm`, `shtml`, `xhtml`, `inc`, `tmpl`, `tpl` |
| Haskell | `hs` |
| Literate Haskell | `lhs` |
| Java Server Page (JSP) | `jsp` |
| Java | `java`, `bsh` |
| JavaDoc | — |
| Java Properties | `properties` |
| JSON | `json`, `sublime-settings`, `sublime-menu`, `sublime-keymap`, `sublime-mousemap`, `sublime-theme`, `sublime-build`, `sublime-project`, `sublime-completions`, `sublime-commands`, `sublime-macro`, `sublime-color-scheme` |
| JavaScript | `js`, `htc` |
| Regular Expressions (Javascript) | — |
| BibTeX | `bib` |
| LaTeX Log | — |
| LaTeX | `tex`, `ltx` |
| TeX | `sty`, `cls` |
| Lisp | `lisp`, `cl`, `clisp`, `l`, `mud`, `el`, `scm`, `ss`, `lsp`, `fasl` |
| Lua | `lua` |
| Make Output | — |
| Makefile | `make`, `GNUmakefile`, `makefile`, `Makefile`, `OCamlMakefile`, `mak`, `mk` |
| Markdown | `md`, `mdown`, `markdown`, `markdn` |
| MultiMarkdown | — |
| MATLAB | `matlab` |
| OCaml | `ml`, `mli` |
| OCamllex | `mll` |
| OCamlyacc | `mly` |
| camlp4 | — |
| Objective-C++ | `mm`, `M`, `h` |
| Objective-C | `m`, `h` |
| PHP Source | — |
| PHP | `php`, `php3`, `php4`, `php5`, `php7`, `phps`, `phpt`, `phtml` |
| Pascal | `pas`, `p`, `dpr` |
| Perl | `pl`, `pm`, `pod`, `t`, `PL` |
| Python | `py`, `py3`, `pyw`, `pyi`, `pyx`, `pyx.in`, `pxd`, `pxd.in`, `pxi`, `pxi.in`, `rpy`, `cpy`, `SConstruct`, `Sconstruct`, `sconstruct`, `SConscript`, `gyp`, `gypi`, `Snakefile`, `wscript` |
| Regular Expressions (Python) | — |
| R Console | — |
| R | `R`, `r`, `s`, `S`, `Rprofile` |
| Rd (R Documentation) | `rd` |
| HTML (Rails) | `rails`, `rhtml`, `erb`, `html.erb` |
| JavaScript (Rails) | `js.erb` |
| Ruby Haml | `haml`, `sass` |
| Ruby on Rails | `rxml`, `builder` |
| SQL (Rails) | `erbsql`, `sql.erb` |
| Regular Expression | `re` |
| reStructuredText | `rst`, `rest` |
| Ruby | `rb`, `Appfile`, `Appraisals`, `Berksfile`, `Brewfile`, `capfile`, `cgi`, `Cheffile`, `config.ru`, `Deliverfile`, `Fastfile`, `fcgi`, `Gemfile`, `gemspec`, `Guardfile`, `irbrc`, `jbuilder`, `podspec`, `prawn`, `rabl`, `rake`, `Rakefile`, `Rantfile`, `rbx`, `rjs`, `ruby.rail`, `Scanfile`, `simplecov`, `Snapfile`, `thor`, `Thorfile`, `Vagrantfile` |
| Cargo Build Results | — |
| Rust | `rs` |
| SQL | `sql`, `ddl`, `dml` |
| Scala | `scala`, `sbt` |
| Bourne Again Shell (bash) | `sh`, `bash`, `zsh`, `fish`, `.bash_aliases`, `.bash_completions`, `.bash_functions`, `.bash_login`, `.bash_logout`, `.bash_profile`, `.bash_variables`, `.bashrc`, `.profile`, `.textmate_init` |
| Shell-Unix-Generic | — |
| commands-builtin-shell-bash | — |
| HTML (Tcl) | `adp` |
| Tcl | `tcl` |
| Textile | `textile` |
| XML | `xml`, `xsd`, `xslt`, `tld`, `dtml`, `rss`, `opml`, `svg` |
| YAML | `yaml`, `yml`, `sublime-syntax` |
| TOML | `toml`, `tml`, `Cargo.lock`, `Gopkg.lock`, `Pipfile`, `pdm.lock`, `poetry.lock`, `uv.lock` |
| Nushell | `nu` |
| PowerShell | `ps1`, `psm1`, `psd1` |
