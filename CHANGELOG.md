# Changelog

All notable changes to this project will be documented in this file.

## [0.9.2] - 2025-01-09

### 🐛 Bug Fixes

- *(cable)* Filter out non-utf8 lines when loading cable candidates (#263)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#261)
- Bump to 0.9.2

## [0.9.1] - 2025-01-09

### 🚀 Features

- *(cli)* Add `no-preview` flag to disable the preview pane (#258)
- *(cable)* Allow custom cable channels to override builtins (#260)

### 🐛 Bug Fixes

- *(fish)* Don't add extra space to prompt if it's an implicit cd (`\.`) (#259)

### 🚜 Refactor

- *(providers)* Improve cable provider files loading sequence (#254)
- *(cable)* Stream in cable results + better error logging + default delimiter consistency (#257)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#251)
- Bump to 0.9.1

## [0.9.0] - 2025-01-07

### 🚀 Features

- *(cable)* Add default git diff cable channel (#226)
- *(channels)* Add support for multi selection (#234)
- *(channels)* Allow sending currently selected entries to other channels (#235)
- Add support for CJK unified ideographs (#243)

### 🐛 Bug Fixes

- *(ingestion)* Use lossy conversion when source doesn't produce valid utf8 (#240)
- *(ansi)* Catch implicit reset escape sequences (#245)

### ⚡ Performance

- Use FxHash instead of SipHash where it makes sense (#237)
- Only display the first 200 log entries when previewing git-repos (#241)
- Drop deduplication when loading cable candidate lines (#248)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#225)
- *(linting)* Add workspace lints (#228)
- Fix linting warnings (#230)
- *(changelog)* Update changelog (auto) (#244)
- Bump to 0.9.0 (#249)

## [0.8.8] - 2025-01-06

### 🚀 Features

- *(ui)* Make background color optional and fallback to terminal default bg color (#219)
- *(ui)* Add new `television` theme that inherits the terminal bg (#220)
- *(ui)* Add support for standard ANSI colors theming and update default theme (#221)

### 🐛 Bug Fixes

- *(cable)* Zsh-history and bash-history cable channels now point to default histfiles locations (#224)

### 🚜 Refactor

- *(ux)* Don't print the list of available channels on channel parsing error (#222)
- *(cable)* More debug information for cable channels (#223)

### 📚 Documentation

- Add shell autocompletion GIF to the README

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#218)
- Bump to 0.8.8

## [0.8.7] - 2025-01-04

### 🐛 Bug Fixes

- *(unix)* Use sed instead of tail for bash and zsh default history channels (#216)

### 🚜 Refactor

- *(shell)* Use $HISTFILE for cable history channels (#210)
- *(cable)* Always create default cable channels in user directory if no cable channels exist (#213)
- *(config)* Check for config file existence before processing subcommands (#214)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#206)
- Bump to 0.8.7 (#217)

## [0.8.6] - 2025-01-01

### 🐛 Bug Fixes

- Automatically create configuration and data directories if they don't exist (#204)
- Nix build (#203)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#202)
- Bump to 0.8.6

## [0.8.5] - 2024-12-31

### 🚀 Features

- *(ui)* Respect BAT_THEME env var for previewer syntax highlighting theme (#201)

### 🐛 Bug Fixes

- *(shell)* Add space if needed when using smart autocomplete (#200)

### 📚 Documentation

- *(config)* Fix typo in default configuration file comment (#198)
- Move parts of README to Wiki (#199)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#197)
- Bump to 0.8.5

## [0.8.4] - 2024-12-31

### 🚀 Features

- *(ux)* Automatically create default user configuration file if nonexistent (#196)

### 🐛 Bug Fixes

- *(channels)* List-channels in kebab-case (#195)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#193)
- Bump version to 0.8.4

## [0.8.3] - 2024-12-30

### 🐛 Bug Fixes

- Bump version to match with the release (#188)

### ⚡ Performance

- *(bin)* Compile binary as a single code unit and use fat LTO (#191)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#187)
- Bump workspace to 0.0.16 (#189)
- Bump to 0.8.3 (#192)

## [0.8.2] - 2024-12-30

### 🚀 Features

- *(shell)* Add separate history binding for zsh integration (#183)
- *(shell)* Add bash support for smart autocomplete and shell history (#184)
- *(shell)* Shell integration support for fish (#186)

### 📚 Documentation

- Add a credits section to the readme (#178)
- Move terminal emulator compatibility section to separate docs file (#179)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#177)

## [0.8.1] - 2024-12-29

### 🐛 Bug Fixes

- *(channels)* Use the number of actual bytes read and not the sample buffer size when calculating the proportion of printable ASCII characters (#174)
- *(ux)* Make DeletePrevWord trigger channel update (#175)

### 📚 Documentation

- Fix broken link in README (#168)
- Update README.md (#171)
- Update readme animations
- Fix broken image in channels.md

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#167)
- Update README.md (#172)
- Bump to 0.8.1 (#176)

## [0.8.0] - 2024-12-29

### 🚀 Features

- *(ui)* Decouple preview title position from input bar position and make it configurable (#144)
- *(shell)* Autocompletion plugin for zsh (#145)
- *(config)* Allow specifying multiple keymaps for the same action + better defaults (#149)
- *(input)* Bind ctrl-w to delete previous word (#150)
- *(ux)* Print current query to stdout on Enter if no entry is selected (#151)
- *(cli)* Allow passing --input <STRING> to prefill input prompt (#153)
- *(cable)* Make preview optional for cable channels (#155)
- *(cable)* Using builtin previewers inside cable channel prototypes (#156)

### 🚜 Refactor

- *(ui)* More compact general layout and make preview panel optional (#148)
- Update default configuration and simplify channel enum conversions (#157)
- *(cable)* Use tail instead of tac for zsh and bash command history channels (#161)

### 📚 Documentation

- Rearrange README, add a features section, and move more technical stuff to separate files (#159)
- Update readme (#160)
- Replace top image with a screenshot of the application (#163)
- Update README with more legible screenshot of the files channel (#164)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#141)
- *(changelog)* Update changelog (auto) (#146)
- *(changelog)* Update changelog (auto) (#154)
- Bump to 0.8.0 (#165)
- Include cable channels (#166)

## [0.7.2] - 2024-12-17

### 🚀 Features

- *(cli)* Add argument to start tv in another working directory (#132)

### 📚 Documentation

- *(readme)* Make channel names consistent everywhere (#138)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#131)

## [0.7.1] - 2024-12-15

### 🚀 Features

- *(channels)* New channel for directories and associated transitions (#130)

### 📚 Documentation

- *(contributing)* Update contributing.md with hot topics and link todo (#129)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#128)

## [0.7.0] - 2024-12-15

### 🚀 Features

- *(themes)* Add support for ui themes (#114)
- *(cable)* Support cable channel invocation through the cli (#116)
- *(themes)* Add support for global themes background colors (#120)
- *(themes)* More builtin UI themes (#125)

### 🐛 Bug Fixes

- *(previewers)* Handle crlf sequences when parsing ansi into ratatui objects (#119)
- *(stdin)* Trim entry newlines when streaming from stdin (#121)
- *(config)* Better handling of default values (#123)

### 🚜 Refactor

- *(screen)* Extract UI related code to separate crate (#106)
- *(help)* Enable help bar by default and add help keybinding (#122)
- *(config)* [**breaking**] Use `$HOME/.config/television` by default for macOS (#124)

### 📚 Documentation

- *(readme)* Add theme previews and udpate readme structure (#126)

### ⚡ Performance

- Add bench for build results list (#107)
- Merge contiguous name match ranges (#108)
- *(ui)* Improve merging of continuous name match ranges (#109)
- Optimize entry ranges (#110)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#105)
- *(version)* Bump workspace to 0.7.0 (#127)

## [0.6.2] - 2024-12-06

### 🐛 Bug Fixes

- *(windows)* Use cmd on windows instead of sh (#102)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#98)
- Use named constant for colors (#99)
- Bump workspace to 0.6.2 (#104)

## [0.6.1] - 2024-12-05

### 🚀 Features

- *(remote)* Distinguish cable channels with a separate icon (#94)

### 🐛 Bug Fixes

- *(cable)* Add cable to unit channel variants (#96)

### 🚜 Refactor

- *(helpbar)* Hide the top help panel by default (#97)

### 📚 Documentation

- *(readme)* Update readme with latest version and fix section link (#93)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#92)

## [0.6.0] - 2024-12-04

### 🚀 Features

- *(layout)* Allow reversing the layout and placing input bar on top (#76)
- *(cable)* Add support for custom channels (#75)

### 🐛 Bug Fixes

- *(output)* Quote output string when it contains spaces and points to an existing path (#77)
- *(stdin)* Better handling of long running stdin streams (#81)
- *(preview)* Remove redundant tokio task when generating builtin file previews (#86)

### 🚜 Refactor

- *(exit)* Use std::process::exit explicitly (#84)

### 📚 Documentation

- *(install)* Update the installation section of the README (#79)
- *(installation)* Update homebrew installation command (#87)

### ⚡ Performance

- Remove unnecessary clone() calls (#83)
- Skip ratatui span when match at end of string (#91)
- Add cache for icon colors (#89)

### ⚙️ Miscellaneous Tasks

- *(changelog)* Update changelog (auto) (#74)
- *(changelog)* Update changelog (auto) (#85)

## [0.5.3] - 2024-11-24

### 🚀 Features

- *(navigation)* Add action to scroll results list by a page (#72)

### 🐛 Bug Fixes

- Quote file names that contain spaces when printing them to stdout (#51)
- *(entry)* Always preserve raw input + match ranges conversions (#62)

### 🚜 Refactor

- *(picker)* Refactor picker logic and add tests to picker, cli, and events (#57)

### 📚 Documentation

- Terminal emulators compatibility and good first issues (#56)
- *(contributing)* Added TOC and Code of Conduct link (#65)

### ⚡ Performance

- *(preview)* Cap the number of concurrent preview tokio tasks in the background (#67)

### 🎨 Styling

- *(git)* Enforce conventional commits on git push with a hook (#61)

### ⚙️ Miscellaneous Tasks

- Add readme version update to github actions (#55)
- *(update_readme)* Fix `update_readme` workflow (#63)
- *(changelog)* Update changelog action trigger (#68)
- *(changelog)* Update changelog (auto) (#70)
- *(changelog)* Update changelog (auto) (#73)
- Bump crate to 0.5.3 and workspace crates to 0.0.7

### Build

- *(infer)* Drop infer dependency and refactor code to a simpler heuristic (#58)

## [0.5.1] - 2024-11-20

### 📚 Documentation

- Add instructions for installing on Arch Linux (#43)
- *(brew)* Add brew installation method for MacOS to README (#45)
- *(config)* Update docs to mention XDG_CONFIG_HOME precedence on all platform (#48)

### ⚙️ Miscellaneous Tasks

- Add CHANGELOG.md (#44)
- *(config)* Default configuration now uses 100% of terminal screen space (#47)
- *(changelog)* Udpate changelog and add corresponding makefile command (#53)
- *(actions)* Remove changelog update from the main branch
- Bump version to 0.5.1

## [0.5.0] - 2024-11-18

### 🚀 Features

- *(cli)* Allow passing passthrough keybindings via stdout for the parent process to deal with (#39)
- *(ui)* Make the top UI help bar toggleable (#41)

### 🚜 Refactor

- *(config)* [**breaking**] Make action names snake case in keybinding configuration (#40)

### 📚 Documentation

- Update README television version
- Update README television version specifier
- Update README television version

### ⚙️ Miscellaneous Tasks

- *(rustfmt)* Update rustfmt.toml (#42)

## [0.4.23] - 2024-11-16

### 🚀 Features

- *(ui)* Make help bar display optional (#35)

### 🚜 Refactor

- *(configuration)* Modularize code and better handling of default options (#32)

### 📚 Documentation

- *(debian)* Add installation docs for debian-based systems (#33)
- *(config)* Update docs default configuration (#34)

## [0.4.22] - 2024-11-16

### 🐛 Bug Fixes

- *(ghactions)* Only trigger cd workflow on new tags (#22)
- *(config)* Swap out default keymaps with user defined ones instead of stacking (#26)

### 🚜 Refactor

- *(channels)* Converting between entries and channels is now generic over channels (#25)

### ⚙️ Miscellaneous Tasks

- *(deb)* Release deb package for television (#31)
- Update CD workflow
- *(cd)* Fix cd configuration for deb packages
- *(cd)* Fix cd configuration for deb packages
- *(versions)* Bump workspace crates versions

## [0.4.21] - 2024-11-13

### 🐛 Bug Fixes

- *(windows)* #20 respect `TELEVISION_CONFIG` env var on windows (#21)

### ⚙️ Miscellaneous Tasks

- *(nix)* Nix flake shell + rust-toolchain.toml setup (#14)

## [0.4.20] - 2024-11-11

### 🐛 Bug Fixes

- *(windows)* Ignore `KeyEventKind::Release` events (#3)
- *(windows)* Bump television_utils to v0.0.1 (#4)
- *(config)* More consistent configuration file location for linux and macos (#9)
- *(workspace)* Fix cargo workspace dependencies
- *(cargo workspace)* Fix cargo workspace structure and dependencies (#15)

### 🚜 Refactor

- *(workspace)* Reorganize cargo workspace (#12)

### 📚 Documentation

- Add terminal emulators compatibility status
- Fix table alignments
- *(readme)* Update terminal emulators compatibility list (#6)

### ⚡ Performance

- *(preview)* Remove temporary plaintext previews in favor of loading message preview (#10)

### ⚙️ Miscellaneous Tasks

- Update README.md install section
- *(coc)* Create CODE_OF_CONDUCT.md (#7)
- *(crate)* Add include directives to Cargo.toml to make the crate leaner (#11)

## [0.4.18] - 2024-11-10

### 🐛 Bug Fixes

- Add the correct permissions to release binaries
- Add `winapi-util` dependency for windows builds

## [0.4.17] - 2024-11-10

### ⚙️ Miscellaneous Tasks

- Udate documentation and dependencies
- Update Makefile and CONTRIBUTING.md
- Testing out the CD pipeline

## [0.4.15] - 2024-11-10

### 🚀 Features

- Send to channel
- More syntaxes and themes for highlighting + configuration

### 🐛 Bug Fixes

- Fixing various issues
- Fixing various issues
- Fix linting issues and ignore derive docs for tests
- Filtering system directories in gitrepos
- Stabilize preview scroll initialization
- Doctests imports
- Gag stdout and stderr while loading theme assets to silence bat warning

### 🚜 Refactor

- Refactoring
- Refactoring matcher
- Extract matcher logic into separate crate
- Split project into separate crates
- More refactoring and fixing doctests

### 📚 Documentation

- Docs and linting
- Documentation
- Update README.md
- Add default keybindings to README.md
- Some work on CONTRIBUTING.md
- More work on CONTRIBUTING.md

### 🧪 Testing

- Tests and docs for strings.rs
- Testing ci

### ⚙️ Miscellaneous Tasks

- Bump version
- Bump version
- Unused imports and ci docs
- Update dependencies and bump version
- Update dependencies and bump version
- Bump version
- Update dependencies and bump version
- Makefile and dist scripts
- *(precommit)* Don't allow committing if clippy doesn't pass
- Patch
- Update workspace crates configurations
- *(previewers)* Unused attributes
- Add license to syntax snippet
- Bump version

<!-- generated by git-cliff -->
