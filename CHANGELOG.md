# Changelog

All notable changes to this project will be documented in this file.
<!-- ignore lint rules that are often triggered by content generated from commits / git-cliff -->
<!-- markdownlint-disable line-length no-bare-urls ul-style emphasis-style -->


## [unreleased]

### ⛰️  Features

- [f887a23](https://github.com/alexpasmantier/television/commit/f887a2390ede0a5f30d61f2bb9d4e1e421109d63) *(cli)* Add a `--ui-scale` [0,100] cli parameter by @alexpasmantier in [#492](https://github.com/alexpasmantier/television/pull/492)

- [cfe49ce](https://github.com/alexpasmantier/television/commit/cfe49ce81c1eb428b7c38fe5b524d67141099946) *(remote)* Redirect `Action::Quit` to `Action::ToggleRemoteControl` when in remote mode by @alexpasmantier in [#508](https://github.com/alexpasmantier/television/pull/508)

- [be8008e](https://github.com/alexpasmantier/television/commit/be8008e97d5ab5063aff27bea52b6315b9f878f7) *(shell)* Improve fish completion system by @lalvarezt in [#494](https://github.com/alexpasmantier/television/pull/494)

- [1086899](https://github.com/alexpasmantier/television/commit/1086899ba76f9b3377a4f67d8d7aef5da2cd310d) *(ui)* Add a UI portrait mode #489 by @cr4ftx in [#496](https://github.com/alexpasmantier/television/pull/496)

### 🐛 Bug Fixes

- [dbff3a3](https://github.com/alexpasmantier/television/commit/dbff3a330b169c422ae384e373b934dceb8e01b2) *(alias)* Move terminal raw mode before loading bat assets #444 by @cr4ftx in [#484](https://github.com/alexpasmantier/television/pull/484)

- [0514a91](https://github.com/alexpasmantier/television/commit/0514a914b630719391d66df61eb9d53c58933c3f) *(alias)* Rename the aliases channel to `alias` by @alexpasmantier in [#485](https://github.com/alexpasmantier/television/pull/485)

- [cd33151](https://github.com/alexpasmantier/television/commit/cd33151bac9422dcef8edcfd16a6553228611631) *(layout)* Double check whether preview is enabled by @nkxxll in [#499](https://github.com/alexpasmantier/television/pull/499)

- [1741a15](https://github.com/alexpasmantier/television/commit/1741a15e526ea0a304bb1cccb5f75bb46d42a6a2) *(preview)* Add a post-processing step to clean out ansi text from non-displayable characters by @alexpasmantier in [#509](https://github.com/alexpasmantier/television/pull/509)

- [1f0c178](https://github.com/alexpasmantier/television/commit/1f0c178a2d79ccf1e6cbe13ea3ec246f987bfbf2) *(results)* Remove keymap hint if help is disabled by @nkxxll in [#480](https://github.com/alexpasmantier/television/pull/480)

- [39dd9ef](https://github.com/alexpasmantier/television/commit/39dd9efd5dfa1fb36281f9f97b753152af82095f) *(shell)* Paste not working in zsh shell integration by @kapobajza in [#512](https://github.com/alexpasmantier/television/pull/512)

### 🚜 Refactor

- [e2f52b8](https://github.com/alexpasmantier/television/commit/e2f52b835d6447c251d7fca6724cf409ed153546) *(cable)* Improve naming and documentation for `prototypes.rs` by @alexpasmantier in [#487](https://github.com/alexpasmantier/television/pull/487)

- [4385317](https://github.com/alexpasmantier/television/commit/4385317e069db287d8d86f987e11e079a7ff6d1c) *(cable)* Split cable related code into separate submodules by @alexpasmantier in [#486](https://github.com/alexpasmantier/television/pull/486)

- [1a5fa5d](https://github.com/alexpasmantier/television/commit/1a5fa5dd4cb485e2b0b08301ca457fa1c6d06094) *(channels)* Some renaming and refactoring the channels module by @alexpasmantier in [#503](https://github.com/alexpasmantier/television/pull/503)

- [b9f42e8](https://github.com/alexpasmantier/television/commit/b9f42e8c29a7eca86a91a6cb00d9c4ee46bb2bd3) *(preview)* Simplify channel previews code and remove intermediate `PreviewKind` struct by @alexpasmantier in [#490](https://github.com/alexpasmantier/television/pull/490)

- [67c067f](https://github.com/alexpasmantier/television/commit/67c067ff40f97eef9090c2a5addca5da50a7fa0f) *(previewer)* A much more efficient preview system for tv by @alexpasmantier in [#506](https://github.com/alexpasmantier/television/pull/506)

- [2b2654b](https://github.com/alexpasmantier/television/commit/2b2654b6aab86707577c0bb5c65301106422e737) *(uncategorized)* Drop TelevisionChannel enum and all associated macros by @alexpasmantier in [#498](https://github.com/alexpasmantier/television/pull/498)

- [cc27b5e](https://github.com/alexpasmantier/television/commit/cc27b5ec6bf3a5a71d6785558e57976db9f2d129) *(uncategorized)* Drop dependency to the `ignore` crate by @alexpasmantier

- [c2f4cc2](https://github.com/alexpasmantier/television/commit/c2f4cc258f5f3b21601e8c7ce98f4584222813b2) *(uncategorized)* Tv no longer needs to write the default cable channel recipes to the user's configuration directory by @alexpasmantier in [#482](https://github.com/alexpasmantier/television/pull/482)

- [67677fb](https://github.com/alexpasmantier/television/commit/67677fb917b6d59d8217eaf6369b95f5ba940ff0) *(uncategorized)* All channels are now cable channels by @alexpasmantier in [#479](https://github.com/alexpasmantier/television/pull/479) [**breaking**]

### 📚 Documentation

- [d3bb3b0](https://github.com/alexpasmantier/television/commit/d3bb3b0a5610b6896a698f89afcf2fb7a2aab44a) *(uncategorized)* Cleanup old todo list by @alexpasmantier in [#483](https://github.com/alexpasmantier/television/pull/483)

### ⚡ Performance

- [fc2f8b9](https://github.com/alexpasmantier/television/commit/fc2f8b9473d1d84712951184da8d4e59edeedc86) *(previews)* Avoid unnecessary preview content copy by @alexpasmantier in [#507](https://github.com/alexpasmantier/television/pull/507)

### ⚙️ Miscellaneous Tasks

- [64c599e](https://github.com/alexpasmantier/television/commit/64c599ef103d18e852d1070c6b313800646f1940) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#491](https://github.com/alexpasmantier/television/pull/491)

- [a602dda](https://github.com/alexpasmantier/television/commit/a602dda34758f9f4a24f1c77b589216c12b9cfba) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#478](https://github.com/alexpasmantier/television/pull/478)

- [0f6b29b](https://github.com/alexpasmantier/television/commit/0f6b29ba817f54da7c6cc694c21127c8588709a0) *(uncategorized)* Add sponsorhips button to the repo by @alexpasmantier



### New Contributors
* @kapobajza made their first contribution in [#512](https://github.com/alexpasmantier/television/pull/512)
* @cr4ftx made their first contribution in [#496](https://github.com/alexpasmantier/television/pull/496)
* @lalvarezt made their first contribution in [#494](https://github.com/alexpasmantier/television/pull/494)


## [0.11.9](https://github.com/alexpasmantier/television/releases/tag/0.11.9) - 2025-04-21

### ⛰️  Features

- [bbbdcb0](https://github.com/alexpasmantier/television/commit/bbbdcb02710ffe656fa49567ecd247813523b557) *(cli)* Add substring matching with `--exact` flag by @nkxxll in [#477](https://github.com/alexpasmantier/television/pull/477)

### ⚡ Performance

- [ce02824](https://github.com/alexpasmantier/television/commit/ce02824f3c4c6a750a30651b478ff255d68ff0de) *(stdin)* Avoid unnecessary allocations when streaming from stdin by @alexpasmantier in [#475](https://github.com/alexpasmantier/television/pull/475)

### ⚙️ Miscellaneous Tasks

- [433d7fa](https://github.com/alexpasmantier/television/commit/433d7fa27057d43be4d9cd6cefd64a79339eb2a6) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#474](https://github.com/alexpasmantier/television/pull/474)

- [f28c18e](https://github.com/alexpasmantier/television/commit/f28c18ed64b50e9be7b95fcfbfd9536837c3ebe3) *(uncategorized)* Release version 0.11.9 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.8...0.11.9


## [0.11.8](https://github.com/alexpasmantier/television/releases/tag/0.11.8) - 2025-04-20

### ⛰️  Features

- [0bd4d42](https://github.com/alexpasmantier/television/commit/0bd4d4274edb00df7847f8be44b952389ad76d1c) *(cli)* Add custom header for input field by @nkxxll in [#472](https://github.com/alexpasmantier/television/pull/472)

### 🐛 Bug Fixes

- [2be2ae7](https://github.com/alexpasmantier/television/commit/2be2ae7cdef9ab356d276271352d284c5ac98ca7) *(unicode)* Add support for more unicode characters by @alexpasmantier in [#470](https://github.com/alexpasmantier/television/pull/470)

### ⚡ Performance

- [a938c1c](https://github.com/alexpasmantier/television/commit/a938c1c46929eb2306bca5201af98c6467be59db) *(ui)* Add frame rate throttling to avoid consuming unnecessary CPU resources by @alexpasmantier in [#473](https://github.com/alexpasmantier/television/pull/473)

### 🧪 Testing

- [315a9f7](https://github.com/alexpasmantier/television/commit/315a9f71fa33910d702ff9e577f582316afc0775) *(benches)* Refactor benches into a simpler and more scalable structure by @alexpasmantier in [#467](https://github.com/alexpasmantier/television/pull/467)

### ⚙️ Miscellaneous Tasks

- [ec8a7db](https://github.com/alexpasmantier/television/commit/ec8a7dbfc37dd6bc6346acbd2b22fb3ab07cdb24) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#465](https://github.com/alexpasmantier/television/pull/465)

- [b4438ab](https://github.com/alexpasmantier/television/commit/b4438ab83585053406a946d242f8760184787d39) *(uncategorized)* Bump to 0.11.8 by @alexpasmantier



### New Contributors
* @nkxxll made their first contribution in [#472](https://github.com/alexpasmantier/television/pull/472)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.7...0.11.8


## [0.11.7](https://github.com/alexpasmantier/television/releases/tag/0.11.7) - 2025-04-12

### ⛰️  Features

- [cb0a46f](https://github.com/alexpasmantier/television/commit/cb0a46fff517e5f3f995ca4828956b6ea1a3ffeb) *(input)* Add action to delete input line by @alexpasmantier in [#464](https://github.com/alexpasmantier/television/pull/464)

- [de6200e](https://github.com/alexpasmantier/television/commit/de6200e45d30e82864434a1d58b4df0081634e14) *(shell)* Support completion in the middle of a prompt by @dkmar in [#450](https://github.com/alexpasmantier/television/pull/450)

### 🐛 Bug Fixes

- [425be1e](https://github.com/alexpasmantier/television/commit/425be1e01b024a0b45433676cb2b05b8820b7f0f) *(ui)* Fix slight responsiveness regression in 0.11.6 by @alexpasmantier in [#461](https://github.com/alexpasmantier/television/pull/461)

### ⚙️ Miscellaneous Tasks

- [e23c307](https://github.com/alexpasmantier/television/commit/e23c3076490c7e480561405d0dab9a4d2d41e890) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#458](https://github.com/alexpasmantier/television/pull/458)

- [6771ecd](https://github.com/alexpasmantier/television/commit/6771ecdde57040f1eb996d9ee4c9c931a18a2c25) *(project)* Migrate from Makefile to Just by @alexpasmantier in [#463](https://github.com/alexpasmantier/television/pull/463)

- [1693584](https://github.com/alexpasmantier/television/commit/169358442b9d619ddbab2367de8934022f4f241c) *(uncategorized)* Bump to 0.11.7 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.6...0.11.7


## [0.11.6](https://github.com/alexpasmantier/television/releases/tag/0.11.6) - 2025-04-09

### ⛰️  Features

- [5bf3d20](https://github.com/alexpasmantier/television/commit/5bf3d20c83d5ea0d5e4c8e146d40f0cc37423611) *(cli)* Add a `--no-help` flag to allow disabling showing the help panel by @alexpasmantier in [#456](https://github.com/alexpasmantier/television/pull/456)

- [b818737](https://github.com/alexpasmantier/television/commit/b81873738a89d1aaa7ac04d67fedf495bc25f062) *(cli)* Add a `--no-remote` flag to lock the application on the cli-invoked channel by @alexpasmantier in [#455](https://github.com/alexpasmantier/television/pull/455)

- [4892dc3](https://github.com/alexpasmantier/television/commit/4892dc3c3c5a5b970b21fb431e6411f6f63e26ab) *(cli)* Add `--select-1` cli flag to automatically select unique result by @alexpasmantier in [#448](https://github.com/alexpasmantier/television/pull/448)

### 🐛 Bug Fixes

- [4a584b4](https://github.com/alexpasmantier/television/commit/4a584b437c413f26c376154fe0751791b9cbd971) *(pwsh)* Use adequate quoting when formatting preview commands for pwsh by @alexpasmantier in [#454](https://github.com/alexpasmantier/television/pull/454)

- [d4913d7](https://github.com/alexpasmantier/television/commit/d4913d73f61d13bdfba67c246845c0849a3caf0c) *(uncategorized)* Silence the `string match` in tv_smart_autocomplete by @dkmar in [#449](https://github.com/alexpasmantier/television/pull/449)

### 🚜 Refactor

- [69c4dcc](https://github.com/alexpasmantier/television/commit/69c4dcc5c57a43bab29f93ce4e14c1cae42d3528) *(debug)* Improve configuration debug logging by @alexpasmantier in [#447](https://github.com/alexpasmantier/television/pull/447)

- [82e3f89](https://github.com/alexpasmantier/television/commit/82e3f890c83eb435c6d0d118a7e72ac30dbb3059) *(passthrough)* Drop support for unused passthrough keybindings by @alexpasmantier in [#446](https://github.com/alexpasmantier/television/pull/446) [**breaking**]

### ⚙️ Miscellaneous Tasks

- [8b267bb](https://github.com/alexpasmantier/television/commit/8b267bb1ff3a005ef6bc20c6ebca952ea99ae0ca) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#443](https://github.com/alexpasmantier/television/pull/443)

- [a008d3f](https://github.com/alexpasmantier/television/commit/a008d3f4b45f85718087a0b203f132416f3a7dc7) *(uncategorized)* Bump to 0.11.6 by @alexpasmantier in [#457](https://github.com/alexpasmantier/television/pull/457)



### New Contributors
* @dkmar made their first contribution in [#449](https://github.com/alexpasmantier/television/pull/449)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.5...0.11.6


## [0.11.5](https://github.com/alexpasmantier/television/releases/tag/0.11.5) - 2025-03-31

### 🐛 Bug Fixes

- [6ba235f](https://github.com/alexpasmantier/television/commit/6ba235fa1193323846f1a956dbcdbe7b98baaa8a) *(results)* Fix alignment for non unit width unicode characters by @alexpasmantier in [#442](https://github.com/alexpasmantier/television/pull/442)

### ⚙️ Miscellaneous Tasks

- [f9a49ac](https://github.com/alexpasmantier/television/commit/f9a49acccf2a667b66081778f851d660b6bb6981) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#436](https://github.com/alexpasmantier/television/pull/436)

- [875b63d](https://github.com/alexpasmantier/television/commit/875b63defee9696335e8d5841092096b44efb5da) *(uncategorized)* Bump to 0.11.5 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.4...0.11.5


## [0.11.4](https://github.com/alexpasmantier/television/releases/tag/0.11.4) - 2025-03-23

### 🚜 Refactor

- [8e17ef6](https://github.com/alexpasmantier/television/commit/8e17ef694e11251faba44069f321b6e5810bd273) *(uncategorized)* Nicer results alignment to improve discoverability of matched patterns by @alexpasmantier in [#435](https://github.com/alexpasmantier/television/pull/435)

### ⚙️ Miscellaneous Tasks

- [b81e0df](https://github.com/alexpasmantier/television/commit/b81e0df791a986990367a6286df4c8798fa7ee11) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#433](https://github.com/alexpasmantier/television/pull/433)

- [5f8b240](https://github.com/alexpasmantier/television/commit/5f8b240c8bd499961c036e58c7d8547e03b6749c) *(uncategorized)* Bump to 0.11.4 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.3...0.11.4


## [0.11.3](https://github.com/alexpasmantier/television/releases/tag/0.11.3) - 2025-03-21

### 🐛 Bug Fixes

- [c573503](https://github.com/alexpasmantier/television/commit/c573503cbfe434ad5927a40e4dcf9be6873bdc72) *(config)* Fix shell integration keybindings not overwriting defaults by @alexpasmantier in [#431](https://github.com/alexpasmantier/television/pull/431)

### ⚙️ Miscellaneous Tasks

- [5752402](https://github.com/alexpasmantier/television/commit/5752402237c23d70d1708add4b3ed523939d4493) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#428](https://github.com/alexpasmantier/television/pull/428)

- [f8bd6c2](https://github.com/alexpasmantier/television/commit/f8bd6c2dd58da37fdda10c61fdb8903f65ec5bce) *(x86)* Add statically linked musl build for x86_64 by @alexpasmantier in [#429](https://github.com/alexpasmantier/television/pull/429)

- [dae6a88](https://github.com/alexpasmantier/television/commit/dae6a8816a73233b26d7f552e0fc3030428ee0d4) *(uncategorized)* Bump to 0.11.3 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.2...0.11.3


## [0.11.2](https://github.com/alexpasmantier/television/releases/tag/0.11.2) - 2025-03-21

### 🐛 Bug Fixes

- [05e3ba3](https://github.com/alexpasmantier/television/commit/05e3ba3b75a94c903d095522e98f8d09250667f0) *(ci)* Fix aarch64 debian builds by @alexpasmantier in [#426](https://github.com/alexpasmantier/television/pull/426)

- [ea6deef](https://github.com/alexpasmantier/television/commit/ea6deef11b68fc3b928d03f20d0ee7fcf0dd15da) *(ci)* More tweaking to the deb releases configuration by @alexpasmantier in [#424](https://github.com/alexpasmantier/television/pull/424)

- [2d74a34](https://github.com/alexpasmantier/television/commit/2d74a3465a506b065e195cae837d743976a67afc) *(ui)* Show preview toggle hint conditionally by @alexpasmantier in [#425](https://github.com/alexpasmantier/television/pull/425)

### ⚙️ Miscellaneous Tasks

- [8ad4a99](https://github.com/alexpasmantier/television/commit/8ad4a9953d7203ae8154f437b6bbd60d86a6c13a) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#423](https://github.com/alexpasmantier/television/pull/423)

- [918dc66](https://github.com/alexpasmantier/television/commit/918dc6686d2ed72d4f9cabfff1ccbfe3eb839467) *(uncategorized)* Bump to 0.11.2 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.1...0.11.2


## [0.11.1](https://github.com/alexpasmantier/television/releases/tag/0.11.1) - 2025-03-20

### 🐛 Bug Fixes

- [4e900d9](https://github.com/alexpasmantier/television/commit/4e900d92c3efa5aa4115dbcc582f96612b26ebd9) *(ci)* Use `fancy-regex` when compiling on arm64 instead of `oniguruma` by @alexpasmantier in [#422](https://github.com/alexpasmantier/television/pull/422)

### ⚙️ Miscellaneous Tasks

- [7069421](https://github.com/alexpasmantier/television/commit/706942183c382df1e8ccb3ab8f50120748dc73b7) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#420](https://github.com/alexpasmantier/television/pull/420)

- [b48127b](https://github.com/alexpasmantier/television/commit/b48127bbba207387bf042f7447de5b5e9e9f11f2) *(uncategorized)* Bump to 0.11.1 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.0...0.11.1


## [0.11.0](https://github.com/alexpasmantier/television/releases/tag/0.11.0) - 2025-03-20

### ⛰️  Features

- [3222037](https://github.com/alexpasmantier/television/commit/3222037a02f229d206af79098e9f9e0e6ab60f00) *(cli)* Allow passing custom keybindings through the cli by @alexpasmantier in [#409](https://github.com/alexpasmantier/television/pull/409)

- [47ea5a2](https://github.com/alexpasmantier/television/commit/47ea5a2b683f40ece608c4b80da56d37831ce581) *(cli)* Allow passing builtin previewers through the cli (e.g. `--preview ':files:'`) by @alexpasmantier in [#403](https://github.com/alexpasmantier/television/pull/403)

- [8eb6ada](https://github.com/alexpasmantier/television/commit/8eb6adafb9272a96fb40eeb3b4a897fd95fec5dd) *(config)* Allow remapping input builtin keybindings (ctrl-e, ctrl-a, etc.) by @alexpasmantier in [#411](https://github.com/alexpasmantier/television/pull/411)

- [d09f670](https://github.com/alexpasmantier/television/commit/d09f6708bc873bf130cabed08958949444f185d8) *(shell)* Add fallback channel to the config for smart autocomplete by @alexpasmantier in [#413](https://github.com/alexpasmantier/television/pull/413)

### 🐛 Bug Fixes

- [731bc7e](https://github.com/alexpasmantier/television/commit/731bc7ee80cd6c98b7461eb6028e31a9936687d9) *(config)* Make user shell integration trigger configuration override defaults entirely by @alexpasmantier in [#417](https://github.com/alexpasmantier/television/pull/417)

- [05bd64a](https://github.com/alexpasmantier/television/commit/05bd64afe9973288ebae07bae326ef3dec2c154a) *(scheduling)* Don't block the main thread when no actions are available to process by @alexpasmantier in [#416](https://github.com/alexpasmantier/television/pull/416)

### 🚜 Refactor

- [7a85728](https://github.com/alexpasmantier/television/commit/7a85728da6adc9b57006902f6e132220da12cddb) *(config)* Simplify keybindings configuration syntax by @alexpasmantier in [#404](https://github.com/alexpasmantier/television/pull/404) [**breaking**]

- [fc2f6cd](https://github.com/alexpasmantier/television/commit/fc2f6cde4624575d08b5b6957bcb81ec6c93e3f0) *(preview)* Improve overall previewer scheduling logic by @alexpasmantier in [#415](https://github.com/alexpasmantier/television/pull/415)

- [3a5b5ec](https://github.com/alexpasmantier/television/commit/3a5b5ec0cca14b8f0c5cddec88e038f90b8ef384) *(startup)* Improve overall startup time and remove first frames artifacts by @alexpasmantier in [#408](https://github.com/alexpasmantier/television/pull/408)

### ⚙️ Miscellaneous Tasks

- [5ee8912](https://github.com/alexpasmantier/television/commit/5ee891230c66119c8544d595b117cef3a5fb7025) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#405](https://github.com/alexpasmantier/television/pull/405)

- [1e4c34f](https://github.com/alexpasmantier/television/commit/1e4c34fecdf2778b06c41ab0e799e890d2185df2) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#402](https://github.com/alexpasmantier/television/pull/402)

- [409d951](https://github.com/alexpasmantier/television/commit/409d951d3c2eeb4afd5d34ee4abbca69e5634241) *(uncategorized)* Bump to 0.11.0 by @alexpasmantier

- [97314d6](https://github.com/alexpasmantier/television/commit/97314d629a03ed1e892c6ea9d8ba0a621019cfe0) *(uncategorized)* Add support for arm64 deb releases by @alexpasmantier in [#412](https://github.com/alexpasmantier/television/pull/412)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.10...0.11.0


## [0.10.10](https://github.com/alexpasmantier/television/releases/tag/0.10.10) - 2025-03-18

### 🐛 Bug Fixes

- [55705c0](https://github.com/alexpasmantier/television/commit/55705c0855238e2683bf6b4d43f7c7094ddf5126) *(zsh)* Use history command to include recent entries by @keturn in [#379](https://github.com/alexpasmantier/television/pull/379)

### 🚜 Refactor

- [ee89b36](https://github.com/alexpasmantier/television/commit/ee89b36b420e2ddc3776afea8658f4930938ebc8) *(stdin)* Disable preview by default and enable when passing `--preview` by @alexpasmantier in [#394](https://github.com/alexpasmantier/television/pull/394)

### 📚 Documentation

- [4f35cc6](https://github.com/alexpasmantier/television/commit/4f35cc6dbdc79925783404a729df24cbd004f954) *(cli)* Improve cli documentation by @alexpasmantier in [#395](https://github.com/alexpasmantier/television/pull/395)

- [0edf224](https://github.com/alexpasmantier/television/commit/0edf224502cc843500ae48e5290a264247930efa) *(man)* Add man pages for tv by @alexpasmantier in [#397](https://github.com/alexpasmantier/television/pull/397)

### ⚙️ Miscellaneous Tasks

- [60ea4a7](https://github.com/alexpasmantier/television/commit/60ea4a7e15bc077bff474a45a572dd523745aa5c) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#391](https://github.com/alexpasmantier/television/pull/391)

- [961ebbb](https://github.com/alexpasmantier/television/commit/961ebbba2bbcbe00d6a989ea04ef6392e7b1aa74) *(uncategorized)* Bump to 0.10.10 by @alexpasmantier



### New Contributors
* @keturn made their first contribution in [#379](https://github.com/alexpasmantier/television/pull/379)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.9...0.10.10


## [0.10.9](https://github.com/alexpasmantier/television/releases/tag/0.10.9) - 2025-03-17

### ⚙️ Miscellaneous Tasks

- [f954e81](https://github.com/alexpasmantier/television/commit/f954e81e4c75d39db15848ff5a076f983a609794) *(deb)* Fix cargo deb output directory error by @alexpasmantier

- [f397dd5](https://github.com/alexpasmantier/television/commit/f397dd5a70539d6702e90a7fd8565542274676a8) *(uncategorized)* Bump to 0.10.9 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.8...0.10.9


## [0.10.8](https://github.com/alexpasmantier/television/releases/tag/0.10.8) - 2025-03-16

### 🐛 Bug Fixes

- [8df4a7a](https://github.com/alexpasmantier/television/commit/8df4a7a2291ab455a6d36dd7018f956da015abf0) *(changelog)* Fix broken links in CHANGELOG.md by @murlakatamenka in [#377](https://github.com/alexpasmantier/television/pull/377)

### 🧪 Testing

- [64b2f73](https://github.com/alexpasmantier/television/commit/64b2f730b3841c32f5d30ae2ae1847db297e8bca) *(uncategorized)* More tests for cli, app, and main by @alexpasmantier in [#375](https://github.com/alexpasmantier/television/pull/375)

### ⚙️ Miscellaneous Tasks

- [d601eb2](https://github.com/alexpasmantier/television/commit/d601eb2c0224018d46c35a0ab0a79803d77ecf4e) *(changelog)* Update changelog (auto) by @github-actions[bot]

- [ed28da3](https://github.com/alexpasmantier/television/commit/ed28da325f09ecb7e9288785fa0dd004f39cfe21) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#371](https://github.com/alexpasmantier/television/pull/371)

- [e26bd59](https://github.com/alexpasmantier/television/commit/e26bd5919f1a81c9ea1564751c2a871f3b628aaa) *(deb)* Temporarily disable old compatibility builds by @alexpasmantier

- [fcf4b35](https://github.com/alexpasmantier/television/commit/fcf4b35272f10488697fb74e493c399e489c6c50) *(uncategorized)* Replace manual convco check and git hook by a dedicated ci step by @alexpasmantier in [#376](https://github.com/alexpasmantier/television/pull/376)



### New Contributors
* @bpairet made their first contribution in [#383](https://github.com/alexpasmantier/television/pull/383)
* @murlakatamenka made their first contribution in [#377](https://github.com/alexpasmantier/television/pull/377)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.7...0.10.8


## [0.10.7](https://github.com/alexpasmantier/television/releases/tag/0.10.7) - 2025-03-06

### ⛰️  Features

- [3441587](https://github.com/alexpasmantier/television/commit/3441587d57879c309e7818ca89375fa2aaced710) *(preview)* Add support for image previews by @I-Azy-I

- [46f5d20](https://github.com/alexpasmantier/television/commit/46f5d20b2c90219b26a17b4803cf9c691c03a461) *(shell)* Allow mapping ctrl-space for builtin shell autocomplete integration by @alexpasmantier

### 🐛 Bug Fixes

- [e2a0fb2](https://github.com/alexpasmantier/television/commit/e2a0fb204724c5b6fc12554a35355a6a419ad198) *(clipboard)* Fix issue where clipboard wouldn't work on X11-based environments by @alexpasmantier in [#364](https://github.com/alexpasmantier/television/pull/364)

### 🚜 Refactor

- [63cb976](https://github.com/alexpasmantier/television/commit/63cb9760272067ab8787085b37690255e20ecbb9) *(ui)* Communicate ui state to tv using channels by @alexpasmantier in [#369](https://github.com/alexpasmantier/television/pull/369)

### 📚 Documentation

- [d47d6f7](https://github.com/alexpasmantier/television/commit/d47d6f7850a92b197f2046d81cf56dd2a1322010) *(uncategorized)* Docs(readme): Update README.md by @alexpasmantier

- [f14c910](https://github.com/alexpasmantier/television/commit/f14c910fb458040fe1f1133ae59be0676d80b374) *(uncategorized)* Update README by @alexpasmantier

### ⚙️ Miscellaneous Tasks

- [4ccb85b](https://github.com/alexpasmantier/television/commit/4ccb85ba5b64b693cb132490365cfb5b339689b1) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#358](https://github.com/alexpasmantier/television/pull/358)

- [5a4c9d3](https://github.com/alexpasmantier/television/commit/5a4c9d329693dcd14c06038cd88f84c72a795744) *(uncategorized)* Bump to 0.10.7 by @alexpasmantier



### New Contributors
* @I-Azy-I made their first contribution

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.6...0.10.7


## [0.10.6](https://github.com/alexpasmantier/television/releases/tag/0.10.6) - 2025-02-08

### 🐛 Bug Fixes

- [8e38ffc](https://github.com/alexpasmantier/television/commit/8e38ffc3ab52414df29d8310e3f7a5b66bb2be6c) *(clipboard)* Gracefully fail if clipboard isn't available by @alexpasmantier in [#350](https://github.com/alexpasmantier/television/pull/350)

- [df2592f](https://github.com/alexpasmantier/television/commit/df2592f2c8aa6edbea0e46a319435e27b8998859) *(events)* Remove sorting and deduplicating incoming actions by @alexpasmantier in [#356](https://github.com/alexpasmantier/television/pull/356)

### 🚜 Refactor

- [7f87b2f](https://github.com/alexpasmantier/television/commit/7f87b2fb31db239b4e534e29989b4286f6a7d052) *(cable)* Use HISTFILE for bash and zsh history channels by @alexpasmantier in [#357](https://github.com/alexpasmantier/television/pull/357)

- [b706dcb](https://github.com/alexpasmantier/television/commit/b706dcb8ddef8b93dca8de21b5e605360b9b8f07) *(help)* Add multiselect keybindings to help panel by @alexpasmantier in [#353](https://github.com/alexpasmantier/television/pull/353)

- [86c100e](https://github.com/alexpasmantier/television/commit/86c100e381b00033f4ae57c53e2070be367333d7) *(ui)* Display current channel in input bar border by @alexpasmantier in [#354](https://github.com/alexpasmantier/television/pull/354)

### 📚 Documentation

- [ade69d7](https://github.com/alexpasmantier/television/commit/ade69d7bfff109141ab0709b4feabc66973c582f) *(uncategorized)* Update readme by @alexpasmantier

- [d40a86d](https://github.com/alexpasmantier/television/commit/d40a86daa281aaa16ef61017f3dad6d899105ed8) *(uncategorized)* Update readme by @alexpasmantier

### ⚙️ Miscellaneous Tasks

- [1e44478](https://github.com/alexpasmantier/television/commit/1e44478147e6d0aa8f320f0b15cd8e4ff4d2f0f9) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#349](https://github.com/alexpasmantier/television/pull/349)

- [11e440c](https://github.com/alexpasmantier/television/commit/11e440c151ef02abc9aed52059c1b648d161ffb5) *(deb)* Add arm64 deb packaging to cd by @alexpasmantier in [#351](https://github.com/alexpasmantier/television/pull/351)

- [bb727bd](https://github.com/alexpasmantier/television/commit/bb727bd070597c60f2750678e9d2cf589ff6f754) *(glibc)* Packaging for older linux distros by @alexpasmantier

- [56be4dc](https://github.com/alexpasmantier/television/commit/56be4dca4f71a21ead8dc50a97e0036ab3ce7b0b) *(winget)* Update winget release configuration by @alexpasmantier

- [28f62f1](https://github.com/alexpasmantier/television/commit/28f62f138dd47c9f0ef3ca33f2daa17a8e9eb909) *(uncategorized)* Bump to 0.10.6 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.5...0.10.6


## [0.10.5](https://github.com/alexpasmantier/television/releases/tag/0.10.5) - 2025-02-07

### 🐛 Bug Fixes

- [4eead98](https://github.com/alexpasmantier/television/commit/4eead98fae18cfc4146def7a776fe4497e1cbc59) *(windows)* Bypass mouse capture disabling on windows by @alexpasmantier in [#348](https://github.com/alexpasmantier/television/pull/348)

### ⚙️ Miscellaneous Tasks

- [fd8bf61](https://github.com/alexpasmantier/television/commit/fd8bf6100963baaf6967cbf983a9ee620effbd4f) *(cd)* Automatically bump winget-pkgs registered version by @kachick in [#340](https://github.com/alexpasmantier/television/pull/340)

- [0d5f394](https://github.com/alexpasmantier/television/commit/0d5f39408279539431f79af3fccc5414e958e50d) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#336](https://github.com/alexpasmantier/television/pull/336)

- [5d552d1](https://github.com/alexpasmantier/television/commit/5d552d1655de46255e6ab62cc8c446bf37ba717d) *(uncategorized)* Bump to 0.10.5 by @alexpasmantier



### New Contributors
* @dependabot[bot] made their first contribution in [#345](https://github.com/alexpasmantier/television/pull/345)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.4...0.10.5


## [0.10.4](https://github.com/alexpasmantier/television/releases/tag/0.10.4) - 2025-02-02

### 🚜 Refactor

- [8881842](https://github.com/alexpasmantier/television/commit/888184235891313cbc3114344d6935e43cb66725) *(shell)* More default shell integration triggers by @alexpasmantier in [#335](https://github.com/alexpasmantier/television/pull/335)

- [a6a73c5](https://github.com/alexpasmantier/television/commit/a6a73c5bb3b23339dfb96538a10f728bb61e1c2d) *(shell)* Improve shell integration configuration syntax by @alexpasmantier in [#334](https://github.com/alexpasmantier/television/pull/334)

### ⚙️ Miscellaneous Tasks

- [c74b47d](https://github.com/alexpasmantier/television/commit/c74b47d07caf12efaf073c16f2177607171c573e) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#330](https://github.com/alexpasmantier/television/pull/330)

- [eaafe40](https://github.com/alexpasmantier/television/commit/eaafe40cfbb7dbf906dad24756a7b2070be33a32) *(uncategorized)* Bump to 0.10.4 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.3...0.10.4


## [0.10.3](https://github.com/alexpasmantier/television/releases/tag/0.10.3) - 2025-01-31

### 🚜 Refactor

- [5214dd1](https://github.com/alexpasmantier/television/commit/5214dd17d0c9b82409dbd81358beb7afc6e28be2) *(app)* Buffering actions and events handling to improve overall UI responsiveness by @alexpasmantier in [#328](https://github.com/alexpasmantier/television/pull/328)

- [be80496](https://github.com/alexpasmantier/television/commit/be804965491b65714613ace52419b9fbb821b9b0) *(draw)* Clearing out mut operations from rendering critical path, avoiding mutexes and perf improvements by @alexpasmantier in [#322](https://github.com/alexpasmantier/television/pull/322)

### ⚙️ Miscellaneous Tasks

- [eaab4e9](https://github.com/alexpasmantier/television/commit/eaab4e966baf1d5dbe83230e4b145ee64fe1b5be) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#320](https://github.com/alexpasmantier/television/pull/320)

- [6955c5b](https://github.com/alexpasmantier/television/commit/6955c5b31357088db4debf202ca99cf303866e7d) *(uncategorized)* Bump to 0.10.3 by @alexpasmantier in [#329](https://github.com/alexpasmantier/television/pull/329)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.2...0.10.3


## [0.10.2](https://github.com/alexpasmantier/television/releases/tag/0.10.2) - 2025-01-26

### 🐛 Bug Fixes

- [f536156](https://github.com/alexpasmantier/television/commit/f536156e7e959fc043dcd972162411bc34b6bc89) *(config)* Add serde default for shell integration configuration by @alexpasmantier in [#319](https://github.com/alexpasmantier/television/pull/319)

### 📚 Documentation

- [4b632f8](https://github.com/alexpasmantier/television/commit/4b632f81f8754b59def555099165d0face28e3c1) *(changelog)* Update changelog template by @alexpasmantier in [#317](https://github.com/alexpasmantier/television/pull/317)

### ⚙️ Miscellaneous Tasks

- [f9f0277](https://github.com/alexpasmantier/television/commit/f9f0277184304f6ddc2d6cb88193273ac8513a5a) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#316](https://github.com/alexpasmantier/television/pull/316)

- [a03da82](https://github.com/alexpasmantier/television/commit/a03da82c56bab5e1e6ed644b82ce8a220a3a6847) *(uncategorized)* Bump to 0.10.2 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.1...0.10.2


## [0.10.1](https://github.com/alexpasmantier/television/releases/tag/0.10.1) - 2025-01-26

### 🐛 Bug Fixes

- [82f471d](https://github.com/alexpasmantier/television/commit/82f471d0aa01285ce82dfb19ab5c81b4b9d1f562) *(cli)* Re-enable clap help feature by @alexpasmantier in [#315](https://github.com/alexpasmantier/television/pull/315)

### ⚙️ Miscellaneous Tasks

- [eede078](https://github.com/alexpasmantier/television/commit/eede07871503b66ad56dbbc66d3f11d491564519) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#312](https://github.com/alexpasmantier/television/pull/312)

- [5271b50](https://github.com/alexpasmantier/television/commit/5271b507a04af992f49ef04871abc8edeb5e0b81) *(terminal)* Custom shell keybindings by @bertrand-chardon in [#313](https://github.com/alexpasmantier/television/pull/313)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.10.0...0.10.1


## [0.10.0](https://github.com/alexpasmantier/television/releases/tag/0.10.0) - 2025-01-25

### ⛰️  Features

- [37b71b7](https://github.com/alexpasmantier/television/commit/37b71b7a881aa634f67c0a051eea5d8a23f66a8b) *(i18n)* Improve support for non-western scripts by @bertrand-chardon in [#305](https://github.com/alexpasmantier/television/pull/305)

### 🐛 Bug Fixes

- [c710904](https://github.com/alexpasmantier/television/commit/c7109044f05dfc967a487ba4583269d3b7b049a5) *(stdout)* Never quote selected entries by @bertrand-chardon in [#307](https://github.com/alexpasmantier/television/pull/307)

- [cb565d6](https://github.com/alexpasmantier/television/commit/cb565d667edeeb629c34f10b50b4a0e78682f643) *(uncategorized)* Add repaint command to the fish shell scripts by @jscarrott in [#303](https://github.com/alexpasmantier/television/pull/303)

### 🚜 Refactor

- [1e8c8db](https://github.com/alexpasmantier/television/commit/1e8c8dbc963c4796b4720ad69e4572c5e881981c) *(uncategorized)* Simplify configuration and build code + leaner crate by @alexpasmantier in [#308](https://github.com/alexpasmantier/television/pull/308)

### ⚡ Performance

- [172ba23](https://github.com/alexpasmantier/television/commit/172ba231eec45b2bff30e80eeca2ccb54504cc01) *(async)* Make overall UI much smoother and snappier by @alexpasmantier in [#311](https://github.com/alexpasmantier/television/pull/311)

### ⚙️ Miscellaneous Tasks

- [4dc7c71](https://github.com/alexpasmantier/television/commit/4dc7c7129f923f937778f66cb512d303fc4df16f) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#294](https://github.com/alexpasmantier/television/pull/294)

- [7a54e5a](https://github.com/alexpasmantier/television/commit/7a54e5a50711f5122f7731863afb85db96816494) *(uncategorized)* Bump to 0.10.0 by @alexpasmantier

- [3970f65](https://github.com/alexpasmantier/television/commit/3970f65946ed2753a1ab0841ea01b45ab23b3fba) *(uncategorized)* Flatten workspace into a single crate by @alexpasmantier in [#306](https://github.com/alexpasmantier/television/pull/306)

- [5750531](https://github.com/alexpasmantier/television/commit/5750531cb2bac6a39aae3348bfc8362a4830fdab) *(uncategorized)* Add zip format in a Windows release assets by @kachick in [#298](https://github.com/alexpasmantier/television/pull/298)



### New Contributors
* @jscarrott made their first contribution in [#303](https://github.com/alexpasmantier/television/pull/303)
* @kachick made their first contribution in [#298](https://github.com/alexpasmantier/television/pull/298)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.9.4...0.10.0


## [0.9.4](https://github.com/alexpasmantier/television/releases/tag/0.9.4) - 2025-01-20

### 🐛 Bug Fixes

- [8bbebf7](https://github.com/alexpasmantier/television/commit/8bbebf7e57600d9f03c607a000188a784728ca11) *(syntect)* Switch back to oniguruma while investigating parsing issues by @alexpasmantier in [#292](https://github.com/alexpasmantier/television/pull/292)

### ⚙️ Miscellaneous Tasks

- [3d97394](https://github.com/alexpasmantier/television/commit/3d973947abeb85312c58f77d146f2a3ae4cb4a09) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#288](https://github.com/alexpasmantier/television/pull/288)

- [40c97c9](https://github.com/alexpasmantier/television/commit/40c97c9c4c5086092f2cfc1bf58b5081e7292f20) *(uncategorized)* Bump workspace to 0.9.4 by @alexpasmantier in [#293](https://github.com/alexpasmantier/television/pull/293)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.9.3...0.9.4


## [0.9.3](https://github.com/alexpasmantier/television/releases/tag/0.9.3) - 2025-01-19

### ⛰️  Features

- [6c3bede](https://github.com/alexpasmantier/television/commit/6c3bede3ca2473d0a9e9d9bd2bc0b42ea9cadbd6) *(preview)* Add support for displaying nerd fonts in preview by @alexpasmantier in [#286](https://github.com/alexpasmantier/television/pull/286)

### 🐛 Bug Fixes

- [c227b2a](https://github.com/alexpasmantier/television/commit/c227b2a20137f615123af5d8d8991d93d8080329) *(cable)* Cable channels now take precedence over builtins for the cli / shell integration by @alexpasmantier in [#278](https://github.com/alexpasmantier/television/pull/278)

### 🚜 Refactor

- [1934d3f](https://github.com/alexpasmantier/television/commit/1934d3f03f4e0398357e1975777670e3e922cabc) *(uncategorized)* Exit application on SIGINT / C-c by @alexpasmantier in [#274](https://github.com/alexpasmantier/television/pull/274)

### 📚 Documentation

- [d68ae21](https://github.com/alexpasmantier/television/commit/d68ae21630bfcfff96b283700a2058d1d44a1f3f) *(readme)* Link to nucleo directly by @archseer in [#266](https://github.com/alexpasmantier/television/pull/266)

### ⚡ Performance

- [a3dc819](https://github.com/alexpasmantier/television/commit/a3dc8196aa5199bedfd62b640c4020a92df9d9d7) *(preview)* Add partial preview rendering and buffer preview requests by @alexpasmantier in [#285](https://github.com/alexpasmantier/television/pull/285)

### ⚙️ Miscellaneous Tasks

- [01a25ac](https://github.com/alexpasmantier/television/commit/01a25ac84623df62e574a3d44cd077224fa6685f) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#265](https://github.com/alexpasmantier/television/pull/265)

- [a43ed22](https://github.com/alexpasmantier/television/commit/a43ed226668d9f2cc1078c66b1e31571ccb22e72) *(uncategorized)* Bump workspace to 0.9.3 by @alexpasmantier in [#287](https://github.com/alexpasmantier/television/pull/287)



### New Contributors
* @archseer made their first contribution in [#266](https://github.com/alexpasmantier/television/pull/266)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.9.2...0.9.3


## [0.9.2](https://github.com/alexpasmantier/television/releases/tag/0.9.2) - 2025-01-09

### 🐛 Bug Fixes

- [9433fea](https://github.com/alexpasmantier/television/commit/9433fea80df9f6277114d2c27795c35450ad7880) *(cable)* Filter out non-utf8 lines when loading cable candidates by @alexpasmantier in [#263](https://github.com/alexpasmantier/television/pull/263)

### ⚙️ Miscellaneous Tasks

- [510b528](https://github.com/alexpasmantier/television/commit/510b52858800cc2b813b21030e9266b0028b1c0a) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#261](https://github.com/alexpasmantier/television/pull/261)

- [1a4dae9](https://github.com/alexpasmantier/television/commit/1a4dae9bd82f284e86ef6e83e07b47dda6e3908f) *(uncategorized)* Bump to 0.9.2 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.9.1...0.9.2


## [0.9.1](https://github.com/alexpasmantier/television/releases/tag/0.9.1) - 2025-01-09

### ⛰️  Features

- [d9ca7b1](https://github.com/alexpasmantier/television/commit/d9ca7b1f9d7460593b3adeac042a50ee3a03649c) *(cable)* Allow custom cable channels to override builtins by @alexpasmantier in [#260](https://github.com/alexpasmantier/television/pull/260)

- [ea8b955](https://github.com/alexpasmantier/television/commit/ea8b955e6d34eade1f83de41805cbab6b7eb6335) *(cli)* Add `no-preview` flag to disable the preview pane by @alexpasmantier in [#258](https://github.com/alexpasmantier/television/pull/258)

### 🐛 Bug Fixes

- [b388a56](https://github.com/alexpasmantier/television/commit/b388a56745f4ad63ded1ebe5f296241695892c4b) *(fish)* Don't add extra space to prompt if it's an implicit cd (`\.`) by @alexpasmantier in [#259](https://github.com/alexpasmantier/television/pull/259)

### 🚜 Refactor

- [3b7fb0c](https://github.com/alexpasmantier/television/commit/3b7fb0c6d6e73a6558a99648c5269ae458ab9404) *(cable)* Stream in cable results + better error logging + default delimiter consistency by @alexpasmantier in [#257](https://github.com/alexpasmantier/television/pull/257)

- [b5e9846](https://github.com/alexpasmantier/television/commit/b5e9846e1b5f62a757057c5403768e20ff3e7f69) *(providers)* Improve cable provider files loading sequence by @alexpasmantier in [#254](https://github.com/alexpasmantier/television/pull/254)

### ⚙️ Miscellaneous Tasks

- [ef26d32](https://github.com/alexpasmantier/television/commit/ef26d326f4f29d01bf9a2087fac7878a7ccbc3db) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#251](https://github.com/alexpasmantier/television/pull/251)

- [d00d8e4](https://github.com/alexpasmantier/television/commit/d00d8e4f84511c3c8c8c3c0ef2634ca671c7c0bd) *(uncategorized)* Bump to 0.9.1 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.9.0...0.9.1


## [0.9.0](https://github.com/alexpasmantier/television/releases/tag/0.9.0) - 2025-01-07

### ⛰️  Features

- [76bff30](https://github.com/alexpasmantier/television/commit/76bff30759612094635cd06366b6eaa240867488) *(cable)* Add default git diff cable channel by @alexpasmantier in [#226](https://github.com/alexpasmantier/television/pull/226)

- [e2398ab](https://github.com/alexpasmantier/television/commit/e2398abcfa6d368389456b79723d87842ee5e33f) *(channels)* Allow sending currently selected entries to other channels by @alexpasmantier in [#235](https://github.com/alexpasmantier/television/pull/235)

- [2e5f65b](https://github.com/alexpasmantier/television/commit/2e5f65baefd7ce10dcb6aa85fd41158f86c6dfcd) *(channels)* Add support for multi selection by @alexpasmantier in [#234](https://github.com/alexpasmantier/television/pull/234)

- [3bd2bb4](https://github.com/alexpasmantier/television/commit/3bd2bb44bd3ab0d4a3423cdb1df3133ed0f4bf84) *(uncategorized)* Add support for CJK unified ideographs by @alexpasmantier in [#243](https://github.com/alexpasmantier/television/pull/243)

### 🐛 Bug Fixes

- [1c00dec](https://github.com/alexpasmantier/television/commit/1c00dece942f09d749699a5d22467b9c279ad950) *(ansi)* Catch implicit reset escape sequences by @alexpasmantier in [#245](https://github.com/alexpasmantier/television/pull/245)

- [a2a264c](https://github.com/alexpasmantier/television/commit/a2a264cc4d7868d31c35ff10912e790cd790262d) *(ingestion)* Use lossy conversion when source doesn't produce valid utf8 by @alexpasmantier in [#240](https://github.com/alexpasmantier/television/pull/240)

### ⚡ Performance

- [8b5beee](https://github.com/alexpasmantier/television/commit/8b5beee1dc3da153d0e4a2c9a9e85ff8540e15d8) *(uncategorized)* Drop deduplication when loading cable candidate lines by @alexpasmantier in [#248](https://github.com/alexpasmantier/television/pull/248)

- [072ecdb](https://github.com/alexpasmantier/television/commit/072ecdba73b4e6677f0ce5d313a45a327df44eed) *(uncategorized)* Only display the first 200 log entries when previewing git-repos by @alexpasmantier in [#241](https://github.com/alexpasmantier/television/pull/241)

- [0624002](https://github.com/alexpasmantier/television/commit/0624002f350d2df0b3aed83c2a8a1b9426757687) *(uncategorized)* Use FxHash instead of SipHash where it makes sense by @alexpasmantier in [#237](https://github.com/alexpasmantier/television/pull/237)

### ⚙️ Miscellaneous Tasks

- [59bdcaa](https://github.com/alexpasmantier/television/commit/59bdcaa278638c97e3ebd469be93d683c15c57fe) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#244](https://github.com/alexpasmantier/television/pull/244)

- [7cd0a9d](https://github.com/alexpasmantier/television/commit/7cd0a9d1b75ecfa9e449e0f8cdcc2663ac9f8d5b) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#225](https://github.com/alexpasmantier/television/pull/225)

- [da2396e](https://github.com/alexpasmantier/television/commit/da2396e19a73ed6b042a78bf037ca7d2894f8946) *(linting)* Add workspace lints by @xosxos in [#228](https://github.com/alexpasmantier/television/pull/228)

- [853da49](https://github.com/alexpasmantier/television/commit/853da494255dcc34d71a6281eee5c353c83bec62) *(uncategorized)* Bump to 0.9.0 by @alexpasmantier in [#249](https://github.com/alexpasmantier/television/pull/249)

- [d207848](https://github.com/alexpasmantier/television/commit/d20784891fc034cf401bcfc6f5f522582d5a8f98) *(uncategorized)* Fix linting warnings by @alexpasmantier in [#230](https://github.com/alexpasmantier/television/pull/230)



### New Contributors
* @xosxos made their first contribution in [#228](https://github.com/alexpasmantier/television/pull/228)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.8...0.9.0


## [0.8.8](https://github.com/alexpasmantier/television/releases/tag/0.8.8) - 2025-01-06

### ⛰️  Features

- [d7e6c35](https://github.com/alexpasmantier/television/commit/d7e6c357357d59152eb198c0d18697d5591ff397) *(ui)* Add support for standard ANSI colors theming and update default theme by @alexpasmantier in [#221](https://github.com/alexpasmantier/television/pull/221)

- [53bd4a3](https://github.com/alexpasmantier/television/commit/53bd4a38159edfec4db7d80813a3cf51a36fb491) *(ui)* Add new `television` theme that inherits the terminal bg by @alexpasmantier in [#220](https://github.com/alexpasmantier/television/pull/220)

- [931a7bb](https://github.com/alexpasmantier/television/commit/931a7bb5c35d992b53f8c4aeee87b66ee9ab14f9) *(ui)* Make background color optional and fallback to terminal default bg color by @alexpasmantier in [#219](https://github.com/alexpasmantier/television/pull/219)

### 🐛 Bug Fixes

- [88b08b7](https://github.com/alexpasmantier/television/commit/88b08b798e5acd39077048ef14e5f33d25067d87) *(cable)* Zsh-history and bash-history cable channels now point to default histfiles locations by @alexpasmantier in [#224](https://github.com/alexpasmantier/television/pull/224)

### 🚜 Refactor

- [3d49d30](https://github.com/alexpasmantier/television/commit/3d49d308c1e2d8c1020bdf27e75bb69cd20e2235) *(cable)* More debug information for cable channels by @alexpasmantier in [#223](https://github.com/alexpasmantier/television/pull/223)

- [074889b](https://github.com/alexpasmantier/television/commit/074889b43fc36d036b067e90a7977a2fd6b519d3) *(ux)* Don't print the list of available channels on channel parsing error by @alexpasmantier in [#222](https://github.com/alexpasmantier/television/pull/222)

### 📚 Documentation

- [21fb3cb](https://github.com/alexpasmantier/television/commit/21fb3cb53cff24b4f30041014c4fa9aa018ba360) *(uncategorized)* Add shell autocompletion GIF to the README by @alexpasmantier

### ⚙️ Miscellaneous Tasks

- [b1309af](https://github.com/alexpasmantier/television/commit/b1309af25f0b5c6741f16b6ef90e084ac2cb9dd8) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#218](https://github.com/alexpasmantier/television/pull/218)

- [6536bbf](https://github.com/alexpasmantier/television/commit/6536bbf32389682b3783a277d176e5e2f4421e60) *(uncategorized)* Bump to 0.8.8 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.7...0.8.8


## [0.8.7](https://github.com/alexpasmantier/television/releases/tag/0.8.7) - 2025-01-04

### 🐛 Bug Fixes

- [3e5f0a4](https://github.com/alexpasmantier/television/commit/3e5f0a44a3405826b599de35f9901dfe4fc86351) *(unix)* Use sed instead of tail for bash and zsh default history channels by @alexpasmantier in [#216](https://github.com/alexpasmantier/television/pull/216)

### 🚜 Refactor

- [657af5e](https://github.com/alexpasmantier/television/commit/657af5e36d82f7e819c592f7dbc2a2c9a41a067d) *(cable)* Always create default cable channels in user directory if no cable channels exist by @alexpasmantier in [#213](https://github.com/alexpasmantier/television/pull/213)

- [124c06c](https://github.com/alexpasmantier/television/commit/124c06c403b019438bbd60663eef48fb8172557c) *(config)* Check for config file existence before processing subcommands by @alexpasmantier in [#214](https://github.com/alexpasmantier/television/pull/214)

- [971a2e7](https://github.com/alexpasmantier/television/commit/971a2e7697d888a09f21fb50a2684e6162ac6329) *(shell)* Use $HISTFILE for cable history channels by @alexpasmantier in [#210](https://github.com/alexpasmantier/television/pull/210)

### ⚙️ Miscellaneous Tasks

- [8089657](https://github.com/alexpasmantier/television/commit/80896578b4f49e346fa5c680d3a486b90d8ec527) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#206](https://github.com/alexpasmantier/television/pull/206)

- [25adee3](https://github.com/alexpasmantier/television/commit/25adee34d8ce35f512cc641c4fc0529545fd2af0) *(uncategorized)* Bump to 0.8.7 by @alexpasmantier in [#217](https://github.com/alexpasmantier/television/pull/217)



### New Contributors
* @tangowithfoxtrot made their first contribution in [#208](https://github.com/alexpasmantier/television/pull/208)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.6...0.8.7


## [0.8.6](https://github.com/alexpasmantier/television/releases/tag/0.8.6) - 2025-01-01

### 🐛 Bug Fixes

- [bff7068](https://github.com/alexpasmantier/television/commit/bff70687814b6dfa682e737d3eec74a918229eb2) *(uncategorized)* Nix build by @tukanoidd in [#203](https://github.com/alexpasmantier/television/pull/203)

- [741ce30](https://github.com/alexpasmantier/television/commit/741ce30b080b462cf8938661ee630a2136b565c5) *(uncategorized)* Automatically create configuration and data directories if they don't exist by @tulilirockz in [#204](https://github.com/alexpasmantier/television/pull/204)

### ⚙️ Miscellaneous Tasks

- [314aa93](https://github.com/alexpasmantier/television/commit/314aa93a4592626cfff56957a62f12f3575d53ae) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#202](https://github.com/alexpasmantier/television/pull/202)

- [df936dd](https://github.com/alexpasmantier/television/commit/df936dd4ebed89d1e7c0fc81892e8230e22aea49) *(uncategorized)* Bump to 0.8.6 by @alexpasmantier



### New Contributors
* @tulilirockz made their first contribution in [#204](https://github.com/alexpasmantier/television/pull/204)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.5...0.8.6


## [0.8.5](https://github.com/alexpasmantier/television/releases/tag/0.8.5) - 2024-12-31

### ⛰️  Features

- [2acfc41](https://github.com/alexpasmantier/television/commit/2acfc41ceb9654e3bb1bf28a51bd9afc2b395293) *(ui)* Respect BAT_THEME env var for previewer syntax highlighting theme by @alexpasmantier in [#201](https://github.com/alexpasmantier/television/pull/201)

### 🐛 Bug Fixes

- [a74dece](https://github.com/alexpasmantier/television/commit/a74deceb982970ae38b6b9052ed65b0deb14c00c) *(shell)* Add space if needed when using smart autocomplete by @alexpasmantier in [#200](https://github.com/alexpasmantier/television/pull/200)

### 📚 Documentation

- [0382ff8](https://github.com/alexpasmantier/television/commit/0382ff81b6e0753448cbfbb94c3ff11ae0253eb3) *(config)* Fix typo in default configuration file comment by @alexpasmantier in [#198](https://github.com/alexpasmantier/television/pull/198)

- [690e88d](https://github.com/alexpasmantier/television/commit/690e88dd1a0ba58d34b1c0db0cfae7577d385df8) *(uncategorized)* Move parts of README to Wiki by @bertrand-chardon in [#199](https://github.com/alexpasmantier/television/pull/199)

### ⚙️ Miscellaneous Tasks

- [d2bf172](https://github.com/alexpasmantier/television/commit/d2bf172f4b029f8eb8b0eaafe4fa556acc93a32b) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#197](https://github.com/alexpasmantier/television/pull/197)

- [8cae592](https://github.com/alexpasmantier/television/commit/8cae59256d0e43a2bf2d1c3ad7db438a9b98a9d8) *(uncategorized)* Bump to 0.8.5 by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.4...0.8.5


## [0.8.4](https://github.com/alexpasmantier/television/releases/tag/0.8.4) - 2024-12-31

### ⛰️  Features

- [343ed3c](https://github.com/alexpasmantier/television/commit/343ed3c126c11452a467cbcaae77bfcf53cd937c) *(ux)* Automatically create default user configuration file if nonexistent by @alexpasmantier in [#196](https://github.com/alexpasmantier/television/pull/196)

### 🐛 Bug Fixes

- [1899873](https://github.com/alexpasmantier/television/commit/1899873680987f797f41dfc682483a4a26ec82b3) *(channels)* List-channels in kebab-case by @fannheyward in [#195](https://github.com/alexpasmantier/television/pull/195)

### ⚙️ Miscellaneous Tasks

- [76da8b0](https://github.com/alexpasmantier/television/commit/76da8b0a5b76d07ae36fe0f972a6f5de549d58a0) *(changelog)* Update changelog (auto) by @github-actions[bot]

- [430e325](https://github.com/alexpasmantier/television/commit/430e3255675139d70a11b1e272d08effb7967ae3) *(uncategorized)* Bump version to 0.8.4 by @alexpasmantier



### New Contributors
* @fannheyward made their first contribution in [#195](https://github.com/alexpasmantier/television/pull/195)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.3...0.8.4


## [0.8.3](https://github.com/alexpasmantier/television/releases/tag/0.8.3) - 2024-12-30

### 🐛 Bug Fixes

- [26036dd](https://github.com/alexpasmantier/television/commit/26036dd0b9663e3aafd2442009b4ff700e841a7a) *(uncategorized)* Bump version to match with the release by @chenrui333 in [#188](https://github.com/alexpasmantier/television/pull/188)

### ⚡ Performance

- [b552657](https://github.com/alexpasmantier/television/commit/b552657926eeac37de24fae5684b1f758fc23f72) *(bin)* Compile binary as a single code unit and use fat LTO by @alexpasmantier in [#191](https://github.com/alexpasmantier/television/pull/191)

### ⚙️ Miscellaneous Tasks

- [9b0129a](https://github.com/alexpasmantier/television/commit/9b0129a8d899c83bc3230cfc36c2266c49b407a8) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#187](https://github.com/alexpasmantier/television/pull/187)

- [0c5da2a](https://github.com/alexpasmantier/television/commit/0c5da2a0c3e72361300b09e03cd2a9fed1619401) *(uncategorized)* Bump to 0.8.3 by @alexpasmantier in [#192](https://github.com/alexpasmantier/television/pull/192)

- [53afed2](https://github.com/alexpasmantier/television/commit/53afed28eebc4be5aab3399cc35a580045033be4) *(uncategorized)* Bump workspace to 0.0.16 by @alexpasmantier in [#189](https://github.com/alexpasmantier/television/pull/189)



### New Contributors
* @chenrui333 made their first contribution in [#188](https://github.com/alexpasmantier/television/pull/188)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.2...0.8.3


## [0.8.2](https://github.com/alexpasmantier/television/releases/tag/0.8.2) - 2024-12-30

### ⛰️  Features

- [b49a069](https://github.com/alexpasmantier/television/commit/b49a06997b93bc48f9cae2a66acda1e4ccfdb621) *(shell)* Shell integration support for fish by @alexpasmantier in [#186](https://github.com/alexpasmantier/television/pull/186)

- [7614fbc](https://github.com/alexpasmantier/television/commit/7614fbc653cd5ec64037a9c5890381ab98269791) *(shell)* Add bash support for smart autocomplete and shell history by @alexpasmantier in [#184](https://github.com/alexpasmantier/television/pull/184)

- [0b5facc](https://github.com/alexpasmantier/television/commit/0b5facca6a3c449dcb7335465b11cae169280612) *(shell)* Add separate history binding for zsh integration by @alexpasmantier in [#183](https://github.com/alexpasmantier/television/pull/183)

### 📚 Documentation

- [537f738](https://github.com/alexpasmantier/television/commit/537f738424ddbfb11d4f840b06b597caf36ecbaa) *(uncategorized)* Move terminal emulator compatibility section to separate docs file by @alexpasmantier in [#179](https://github.com/alexpasmantier/television/pull/179)

- [c3d6b87](https://github.com/alexpasmantier/television/commit/c3d6b873d0f5a0ef25087dd09e725dfa4b7ad055) *(uncategorized)* Add a credits section to the readme by @alexpasmantier in [#178](https://github.com/alexpasmantier/television/pull/178)

### ⚙️ Miscellaneous Tasks

- [d8eac4d](https://github.com/alexpasmantier/television/commit/d8eac4da8a738ba6c888874f8c0069d55cd236af) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#177](https://github.com/alexpasmantier/television/pull/177)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.1...0.8.2


## [0.8.1](https://github.com/alexpasmantier/television/releases/tag/0.8.1) - 2024-12-29

### 🐛 Bug Fixes

- [08fa41b](https://github.com/alexpasmantier/television/commit/08fa41b06c59cc0fc1e0fcc8803a4f77517190b1) *(channels)* Use the number of actual bytes read and not the sample buffer size when calculating the proportion of printable ASCII characters by @alexpasmantier in [#174](https://github.com/alexpasmantier/television/pull/174)

- [97343c6](https://github.com/alexpasmantier/television/commit/97343c679d5fd93548226ba34c7c8fd3d52137c9) *(ux)* Make DeletePrevWord trigger channel update by @alexpasmantier in [#175](https://github.com/alexpasmantier/television/pull/175)

### 📚 Documentation

- [b74b130](https://github.com/alexpasmantier/television/commit/b74b13075df34cad63b0a45e5face1f240cfa408) *(uncategorized)* Fix broken image in channels.md by @alexpasmantier

- [dc4028f](https://github.com/alexpasmantier/television/commit/dc4028fd7cf0c697083a28d2bd949e00bd022a0b) *(uncategorized)* Update readme animations by @alexpasmantier

- [a14dccb](https://github.com/alexpasmantier/television/commit/a14dccb726cd09d43811201e80768d51f0bb8d38) *(uncategorized)* Update README.md by @alexpasmantier in [#171](https://github.com/alexpasmantier/television/pull/171)

- [90c2b9c](https://github.com/alexpasmantier/television/commit/90c2b9ce437535f50f0a431a6629e8fc006a2f1d) *(uncategorized)* Fix broken link in README by @alexpasmantier in [#168](https://github.com/alexpasmantier/television/pull/168)

### ⚙️ Miscellaneous Tasks

- [19e6593](https://github.com/alexpasmantier/television/commit/19e6593968c3b15a77286e90ee201305359ee8f2) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#167](https://github.com/alexpasmantier/television/pull/167)

- [7434f14](https://github.com/alexpasmantier/television/commit/7434f1476abeaeb71d135389bd02092d68b36446) *(uncategorized)* Bump to 0.8.1 by @alexpasmantier in [#176](https://github.com/alexpasmantier/television/pull/176)

- [e9c3ebf](https://github.com/alexpasmantier/television/commit/e9c3ebf05f66060f51b1c75b90e3f7b8af137575) *(uncategorized)* Docs(readme): Update README.md by @bertrand-chardon in [#172](https://github.com/alexpasmantier/television/pull/172)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.8.0...0.8.1


## [0.8.0](https://github.com/alexpasmantier/television/releases/tag/0.8.0) - 2024-12-29

### ⛰️  Features

- [ee71e47](https://github.com/alexpasmantier/television/commit/ee71e4788f8ee3f6fd3891e6c0316a4a4df7b369) *(cable)* Using builtin previewers inside cable channel prototypes by @alexpasmantier in [#156](https://github.com/alexpasmantier/television/pull/156)

- [e034615](https://github.com/alexpasmantier/television/commit/e0346155945250defd3298a61aa3f6fee1518283) *(cable)* Make preview optional for cable channels by @alexpasmantier in [#155](https://github.com/alexpasmantier/television/pull/155)

- [309ff53](https://github.com/alexpasmantier/television/commit/309ff537a499a0d9350c907735b07bdb016d7538) *(cli)* Allow passing --input <STRING> to prefill input prompt by @alexpasmantier in [#153](https://github.com/alexpasmantier/television/pull/153)

- [557686e](https://github.com/alexpasmantier/television/commit/557686e1976ef474de314c790270985d6c7c73af) *(config)* Allow specifying multiple keymaps for the same action + better defaults by @alexpasmantier in [#149](https://github.com/alexpasmantier/television/pull/149)

- [12fdf94](https://github.com/alexpasmantier/television/commit/12fdf94e5de7abff4792db760ca77f7223d6f438) *(input)* Bind ctrl-w to delete previous word by @alexpasmantier in [#150](https://github.com/alexpasmantier/television/pull/150)

- [68d1189](https://github.com/alexpasmantier/television/commit/68d118986cbed4d86ccc3006ce5244a358f244ee) *(shell)* Autocompletion plugin for zsh by @alexpasmantier in [#145](https://github.com/alexpasmantier/television/pull/145)

- [22f1b4d](https://github.com/alexpasmantier/television/commit/22f1b4dc337353782474bf59580cab91b87f9ede) *(ui)* Decouple preview title position from input bar position and make it configurable by @alexpasmantier in [#144](https://github.com/alexpasmantier/television/pull/144)

- [c3b8c68](https://github.com/alexpasmantier/television/commit/c3b8c68d1bb5b7d4351f66af125af1561dccf248) *(ux)* Print current query to stdout on Enter if no entry is selected by @alexpasmantier in [#151](https://github.com/alexpasmantier/television/pull/151)

### 🚜 Refactor

- [157d01c](https://github.com/alexpasmantier/television/commit/157d01c4e71faaaa106f922e9a3b59139d632003) *(cable)* Use tail instead of tac for zsh and bash command history channels by @alexpasmantier in [#161](https://github.com/alexpasmantier/television/pull/161)

- [499bfdb](https://github.com/alexpasmantier/television/commit/499bfdb8e5b33d1c4c8554908fc3d71abf8bd0b3) *(ui)* More compact general layout and make preview panel optional by @alexpasmantier in [#148](https://github.com/alexpasmantier/television/pull/148)

- [697f295](https://github.com/alexpasmantier/television/commit/697f295afb930298f8e37e536ce89a573b863a29) *(uncategorized)* Update default configuration and simplify channel enum conversions by @alexpasmantier in [#157](https://github.com/alexpasmantier/television/pull/157)

### 📚 Documentation

- [8de82fe](https://github.com/alexpasmantier/television/commit/8de82fec5d2bea58ef8f74f0c042088b62ec2a01) *(uncategorized)* Update README with more legible screenshot of the files channel by @alexpasmantier in [#164](https://github.com/alexpasmantier/television/pull/164)

- [07a7c7b](https://github.com/alexpasmantier/television/commit/07a7c7b34c87e0e4cb70ce4fff521b70c5b549f2) *(uncategorized)* Replace top image with a screenshot of the application by @alexpasmantier in [#163](https://github.com/alexpasmantier/television/pull/163)

- [f83c5d1](https://github.com/alexpasmantier/television/commit/f83c5d1396664fae4d68ed26c7b6dbc60f507bea) *(uncategorized)* Update readme by @alexpasmantier in [#160](https://github.com/alexpasmantier/television/pull/160)

- [6d706b4](https://github.com/alexpasmantier/television/commit/6d706b4c12bfeae2bb097fe75deb17f3e0fcdcb0) *(uncategorized)* Rearrange README, add a features section, and move more technical stuff to separate files by @alexpasmantier in [#159](https://github.com/alexpasmantier/television/pull/159)

### ⚙️ Miscellaneous Tasks

- [3f92ca2](https://github.com/alexpasmantier/television/commit/3f92ca2b135205c7112f0e9e2bb36f8f4866dccc) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#154](https://github.com/alexpasmantier/television/pull/154)

- [ba5b085](https://github.com/alexpasmantier/television/commit/ba5b0857c3ce54a6fe37ca6e7d6824114188d8b7) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#146](https://github.com/alexpasmantier/television/pull/146)

- [ac7762e](https://github.com/alexpasmantier/television/commit/ac7762e8f2d7a2c5d582be5b20fe2f8f22a71234) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#141](https://github.com/alexpasmantier/television/pull/141)

- [f707190](https://github.com/alexpasmantier/television/commit/f7071904397b03f25f8e56df1d5ca2f5bc445fd9) *(uncategorized)* Include cable channels by @alexpasmantier in [#166](https://github.com/alexpasmantier/television/pull/166)

- [1bc6f12](https://github.com/alexpasmantier/television/commit/1bc6f127821bdaa93291a04afaf19111737ee42f) *(uncategorized)* Bump to 0.8.0 by @alexpasmantier in [#165](https://github.com/alexpasmantier/television/pull/165)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.7.2...0.8.0


## [0.7.2](https://github.com/alexpasmantier/television/releases/tag/0.7.2) - 2024-12-17

### ⛰️  Features

- [882737d](https://github.com/alexpasmantier/television/commit/882737d147ce64bb50f2193a0e47bb10fd2970d8) *(cli)* Add argument to start tv in another working directory by @defigli in [#132](https://github.com/alexpasmantier/television/pull/132)

### 📚 Documentation

- [e27c834](https://github.com/alexpasmantier/television/commit/e27c8342e84b195027202b8c92a5e694f0ea6d46) *(readme)* Make channel names consistent everywhere by @peter-fh in [#138](https://github.com/alexpasmantier/television/pull/138)

### ⚙️ Miscellaneous Tasks

- [3b8ab1f](https://github.com/alexpasmantier/television/commit/3b8ab1fbd8416bcdf774421352eccf5b53752b05) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#131](https://github.com/alexpasmantier/television/pull/131)



### New Contributors
* @peter-fh made their first contribution in [#138](https://github.com/alexpasmantier/television/pull/138)
* @defigli made their first contribution in [#132](https://github.com/alexpasmantier/television/pull/132)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.7.1...0.7.2


## [0.7.1](https://github.com/alexpasmantier/television/releases/tag/0.7.1) - 2024-12-15

### ⛰️  Features

- [18c5213](https://github.com/alexpasmantier/television/commit/18c5213e83955e3a58fc50cf6d948bb93af2c2c0) *(channels)* New channel for directories and associated transitions by @alexpasmantier in [#130](https://github.com/alexpasmantier/television/pull/130)

### 📚 Documentation

- [c0c790c](https://github.com/alexpasmantier/television/commit/c0c790cb48011a7ff055d71779ebad3ac20b6f91) *(contributing)* Update contributing.md with hot topics and link todo by @alexpasmantier in [#129](https://github.com/alexpasmantier/television/pull/129)

### ⚙️ Miscellaneous Tasks

- [7fa469a](https://github.com/alexpasmantier/television/commit/7fa469aea02c7c23d2ebf953c8b8c6ad2d39d3ec) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#128](https://github.com/alexpasmantier/television/pull/128)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.7.0...0.7.1


## [0.7.0](https://github.com/alexpasmantier/television/releases/tag/0.7.0) - 2024-12-15

### ⛰️  Features

- [937d0f0](https://github.com/alexpasmantier/television/commit/937d0f0758367eb209f5abfff2ef7afdc09d4971) *(cable)* Support cable channel invocation through the cli by @alexpasmantier in [#116](https://github.com/alexpasmantier/television/pull/116)

- [4164e90](https://github.com/alexpasmantier/television/commit/4164e9092b577f577ada87286326b465f07300f6) *(themes)* More builtin UI themes by @alexpasmantier in [#125](https://github.com/alexpasmantier/television/pull/125)

- [11da96d](https://github.com/alexpasmantier/television/commit/11da96d7fb1d380a289e33482bd534e1cd4fa4cd) *(themes)* Add support for global themes background colors by @alexpasmantier in [#120](https://github.com/alexpasmantier/television/pull/120)

- [913aa85](https://github.com/alexpasmantier/television/commit/913aa85af03ad1b819f58388c8f0192b6d3e6b66) *(themes)* Add support for ui themes by @alexpasmantier in [#114](https://github.com/alexpasmantier/television/pull/114)

### 🐛 Bug Fixes

- [7b114b7](https://github.com/alexpasmantier/television/commit/7b114b7cb6c7559c98546451461e8af5da4fb645) *(config)* Better handling of default values by @alexpasmantier in [#123](https://github.com/alexpasmantier/television/pull/123)

- [ea752b1](https://github.com/alexpasmantier/television/commit/ea752b13e6e2933a0be785cf29a9a7ebac123a23) *(previewers)* Handle crlf sequences when parsing ansi into ratatui objects by @alexpasmantier in [#119](https://github.com/alexpasmantier/television/pull/119)

- [9809e74](https://github.com/alexpasmantier/television/commit/9809e742d86443950800854042013ae80094584e) *(stdin)* Trim entry newlines when streaming from stdin by @alexpasmantier in [#121](https://github.com/alexpasmantier/television/pull/121)

### 🚜 Refactor

- [a7064c1](https://github.com/alexpasmantier/television/commit/a7064c18c8a74a0eba2d93be904c7f72bbff1e1c) *(config)* Use `$HOME/.config/television` by default for macOS by @alexpasmantier in [#124](https://github.com/alexpasmantier/television/pull/124) [**breaking**]

- [37b2dda](https://github.com/alexpasmantier/television/commit/37b2dda7297a83f58d35d71de5cb971a355ff3f7) *(help)* Enable help bar by default and add help keybinding by @alexpasmantier in [#122](https://github.com/alexpasmantier/television/pull/122)

- [54399e3](https://github.com/alexpasmantier/television/commit/54399e377776ae6a192d4565647a412e3e49354e) *(screen)* Extract UI related code to separate crate by @alexpasmantier in [#106](https://github.com/alexpasmantier/television/pull/106)

### 📚 Documentation

- [630e791](https://github.com/alexpasmantier/television/commit/630e791961767ae071b883728e901dd201c376bb) *(readme)* Add theme previews and udpate readme structure by @alexpasmantier in [#126](https://github.com/alexpasmantier/television/pull/126)

### ⚡ Performance

- [758bfc2](https://github.com/alexpasmantier/television/commit/758bfc290a09f708b1f7bcab915cc0465aaa8af8) *(ui)* Improve merging of continuous name match ranges by @alexpasmantier in [#109](https://github.com/alexpasmantier/television/pull/109)

- [a4d15af](https://github.com/alexpasmantier/television/commit/a4d15af694cb09a2bf338ea7b6b573d274cdeddb) *(uncategorized)* Optimize entry ranges by @bertrand-chardon in [#110](https://github.com/alexpasmantier/television/pull/110)

- [5fb02c7](https://github.com/alexpasmantier/television/commit/5fb02c768f82d81af2426661b67183dbc333b21d) *(uncategorized)* Merge contiguous name match ranges by @bertrand-chardon in [#108](https://github.com/alexpasmantier/television/pull/108)

- [c0db566](https://github.com/alexpasmantier/television/commit/c0db566a48d7821dcdc4bd9ff330b24b8df6b963) *(uncategorized)* Add bench for build results list by @bertrand-chardon in [#107](https://github.com/alexpasmantier/television/pull/107)

### ⚙️ Miscellaneous Tasks

- [6e35e1a](https://github.com/alexpasmantier/television/commit/6e35e1a50ce4ace43920db8eba459c9de965f05a) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#105](https://github.com/alexpasmantier/television/pull/105)

- [a8e3ea5](https://github.com/alexpasmantier/television/commit/a8e3ea5f8954e2cde8c81c10c4cf5172ab2a00f1) *(version)* Bump workspace to 0.7.0 by @alexpasmantier in [#127](https://github.com/alexpasmantier/television/pull/127)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.6.2...0.7.0


## [0.6.2](https://github.com/alexpasmantier/television/releases/tag/0.6.2) - 2024-12-06

### 🐛 Bug Fixes

- [f9d33e4](https://github.com/alexpasmantier/television/commit/f9d33e4797e6d21bf27de62d51ecd8985455a5a2) *(windows)* Use cmd on windows instead of sh by @Liyixin95 in [#102](https://github.com/alexpasmantier/television/pull/102)

### ⚙️ Miscellaneous Tasks

- [2ea6f9a](https://github.com/alexpasmantier/television/commit/2ea6f9a5c1a2c84b03cf390e02df0647d7de271d) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#98](https://github.com/alexpasmantier/television/pull/98)

- [ffc8dae](https://github.com/alexpasmantier/television/commit/ffc8dae4942102a9ec4c8661d6a0adfb1f4813fc) *(uncategorized)* Bump workspace to 0.6.2 by @alexpasmantier in [#104](https://github.com/alexpasmantier/television/pull/104)

- [4567f26](https://github.com/alexpasmantier/television/commit/4567f26a37995f9af6648777ada491c227bcaccd) *(uncategorized)* Use named constant for colors by @bertrand-chardon in [#99](https://github.com/alexpasmantier/television/pull/99)



### New Contributors
* @Liyixin95 made their first contribution in [#102](https://github.com/alexpasmantier/television/pull/102)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.6.1...0.6.2


## [0.6.1](https://github.com/alexpasmantier/television/releases/tag/0.6.1) - 2024-12-05

### ⛰️  Features

- [ad3e52d](https://github.com/alexpasmantier/television/commit/ad3e52d3407a25fff6a2a86f64de46a5fd8b89fd) *(remote)* Distinguish cable channels with a separate icon by @alexpasmantier in [#94](https://github.com/alexpasmantier/television/pull/94)

### 🐛 Bug Fixes

- [795db19](https://github.com/alexpasmantier/television/commit/795db19ffffafb080a54b6fc8d699f9c9d316255) *(cable)* Add cable to unit channel variants by @alexpasmantier in [#96](https://github.com/alexpasmantier/television/pull/96)

### 🚜 Refactor

- [6a13590](https://github.com/alexpasmantier/television/commit/6a1359055dc9546c235f6470deabf9dbaa0f8e61) *(helpbar)* Hide the top help panel by default by @alexpasmantier in [#97](https://github.com/alexpasmantier/television/pull/97)

### 📚 Documentation

- [b6f12b3](https://github.com/alexpasmantier/television/commit/b6f12b372b85c571539989d73b4bbfec6f548541) *(readme)* Update readme with latest version and fix section link by @alexpasmantier in [#93](https://github.com/alexpasmantier/television/pull/93)

### ⚙️ Miscellaneous Tasks

- [99a4405](https://github.com/alexpasmantier/television/commit/99a4405e66a624494ec69afbd94f19f9d2dc31a1) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#92](https://github.com/alexpasmantier/television/pull/92)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.6.0...0.6.1


## [0.6.0](https://github.com/alexpasmantier/television/releases/tag/0.6.0) - 2024-12-04

### ⛰️  Features

- [a5f5d20](https://github.com/alexpasmantier/television/commit/a5f5d20071a3d58761c1917b34fcd0a12ae7f102) *(cable)* Add support for custom channels by @alexpasmantier in [#75](https://github.com/alexpasmantier/television/pull/75)

- [2206711](https://github.com/alexpasmantier/television/commit/220671106e621454e2088ccf08bc9957f240bbec) *(layout)* Allow reversing the layout and placing input bar on top by @alexpasmantier in [#76](https://github.com/alexpasmantier/television/pull/76)

### 🐛 Bug Fixes

- [1ebec7e](https://github.com/alexpasmantier/television/commit/1ebec7ead22e2bac806450f8a3ab31e840838a4c) *(output)* Quote output string when it contains spaces and points to an existing path by @alexpasmantier in [#77](https://github.com/alexpasmantier/television/pull/77)

- [128a611](https://github.com/alexpasmantier/television/commit/128a6116c3e7ffb1f850bae309c84b2da43f3d77) *(preview)* Remove redundant tokio task when generating builtin file previews by @alexpasmantier in [#86](https://github.com/alexpasmantier/television/pull/86)

- [d3c16af](https://github.com/alexpasmantier/television/commit/d3c16af4e94e2f47b9e966b8bd6284392368a37b) *(stdin)* Better handling of long running stdin streams by @alexpasmantier in [#81](https://github.com/alexpasmantier/television/pull/81)

### 🚜 Refactor

- [30f1940](https://github.com/alexpasmantier/television/commit/30f194081514d25a3a4e8a13e092cc6c3e896736) *(exit)* Use std::process::exit explicitly by @alexpasmantier in [#84](https://github.com/alexpasmantier/television/pull/84)

### 📚 Documentation

- [48ea12e](https://github.com/alexpasmantier/television/commit/48ea12ed7a0f273cf9154b4b3e3aeb2ce5e5add0) *(install)* Update the installation section of the README by @alexpasmantier in [#79](https://github.com/alexpasmantier/television/pull/79)

- [20cf83b](https://github.com/alexpasmantier/television/commit/20cf83b72017bec4029fd502b7c730e1bc99dd31) *(installation)* Update homebrew installation command by @alexpasmantier in [#87](https://github.com/alexpasmantier/television/pull/87)

### ⚡ Performance

- [fee4ed2](https://github.com/alexpasmantier/television/commit/fee4ed2671be1aee9c6f3fd2c77d45c208525c83) *(uncategorized)* Add cache for icon colors by @bertrand-chardon in [#89](https://github.com/alexpasmantier/television/pull/89)

- [b7ddb00](https://github.com/alexpasmantier/television/commit/b7ddb00c4eadacfb5512819798072f112b0bbb07) *(uncategorized)* Skip ratatui span when match at end of string by @bertrand-chardon in [#91](https://github.com/alexpasmantier/television/pull/91)

- [4bea114](https://github.com/alexpasmantier/television/commit/4bea114635848e1d26a2226585981e37fd707843) *(uncategorized)* Remove unnecessary clone() calls by @bertrand-chardon

### ⚙️ Miscellaneous Tasks

- [c96d855](https://github.com/alexpasmantier/television/commit/c96d85529033cb509e38114c5c14c3e7ff877cb8) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#85](https://github.com/alexpasmantier/television/pull/85)

- [9998b9d](https://github.com/alexpasmantier/television/commit/9998b9d9f80d381e58353236194f2cd511596aa9) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#74](https://github.com/alexpasmantier/television/pull/74)



### New Contributors
* @moritzwilksch made their first contribution in [#78](https://github.com/alexpasmantier/television/pull/78)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.5.3...0.6.0


## [0.5.3](https://github.com/alexpasmantier/television/releases/tag/0.5.3) - 2024-11-24

### ⛰️  Features

- [6d39651](https://github.com/alexpasmantier/television/commit/6d3965152e91639babaedb1e8a00953a9b01b05f) *(navigation)* Add action to scroll results list by a page by @alexpasmantier in [#72](https://github.com/alexpasmantier/television/pull/72)

### 🐛 Bug Fixes

- [edd9df4](https://github.com/alexpasmantier/television/commit/edd9df4e2911e1fd8e96a83e9f4696f61b0f5647) *(entry)* Always preserve raw input + match ranges conversions by @alexpasmantier in [#62](https://github.com/alexpasmantier/television/pull/62)

- [21cdaae](https://github.com/alexpasmantier/television/commit/21cdaaee42fade21f43014c983bb650352f61926) *(uncategorized)* Quote file names that contain spaces when printing them to stdout by @fredmorcos in [#51](https://github.com/alexpasmantier/television/pull/51)

### 🚜 Refactor

- [b757305](https://github.com/alexpasmantier/television/commit/b757305d7ab8d3ca7059b2a0b603215c8f9a608a) *(picker)* Refactor picker logic and add tests to picker, cli, and events by @alexpasmantier in [#57](https://github.com/alexpasmantier/television/pull/57)

### 📚 Documentation

- [790c870](https://github.com/alexpasmantier/television/commit/790c870ff39e6c41442706cbc9bc8f24af73c9fe) *(contributing)* Added TOC and Code of Conduct link by @MohamedBsh

- [cdcce4d](https://github.com/alexpasmantier/television/commit/cdcce4d9f9afcf852c024f7d54f05a55c3147ddd) *(uncategorized)* Terminal emulators compatibility and good first issues by @alexpasmantier in [#56](https://github.com/alexpasmantier/television/pull/56)

### ⚡ Performance

- [84d54b5](https://github.com/alexpasmantier/television/commit/84d54b5751611684d30ff287a89a681410b2be84) *(preview)* Cap the number of concurrent preview tokio tasks in the background by @alexpasmantier in [#67](https://github.com/alexpasmantier/television/pull/67)

### 🎨 Styling

- [b703e1b](https://github.com/alexpasmantier/television/commit/b703e1b26c9d9816da297f2b8744a22139635f04) *(git)* Enforce conventional commits on git push with a hook by @alexpasmantier in [#61](https://github.com/alexpasmantier/television/pull/61)

### ⚙️ Miscellaneous Tasks

- [ebcccb1](https://github.com/alexpasmantier/television/commit/ebcccb146a3fb1e0290d3649adf71d8b9f984f35) *(changelog)* Update changelog (auto) by @github-actions[bot] in [#73](https://github.com/alexpasmantier/television/pull/73)

- [c87af47](https://github.com/alexpasmantier/television/commit/c87af47d4e7cec67c5e844cc77849cedb5037bfa) *(changelog)* Update changelog (auto) by @github-actions[bot]

- [03fb7d0](https://github.com/alexpasmantier/television/commit/03fb7d0f35740707a3c2612a10f0b3ff5914589c) *(changelog)* Update changelog action trigger by @alexpasmantier

- [dc36b21](https://github.com/alexpasmantier/television/commit/dc36b2152d50c377e7c0741112e8038c464f04fc) *(update_readme)* Fix `update_readme` workflow by @alexpasmantier

- [2fc9bd9](https://github.com/alexpasmantier/television/commit/2fc9bd9e80797905feea5e6109d398f5a587bb1c) *(uncategorized)* Bump crate to 0.5.3 and workspace crates to 0.0.7 by @alexpasmantier

- [0f6aad9](https://github.com/alexpasmantier/television/commit/0f6aad952f2793bb636c148ea472440daba166a2) *(uncategorized)* Add readme version update to github actions by @alexpasmantier in [#55](https://github.com/alexpasmantier/television/pull/55)

### Build

- [f0e1115](https://github.com/alexpasmantier/television/commit/f0e1115bab72a0226f728ae17ac1937d2c7d010d) *(infer)* Drop infer dependency and refactor code to a simpler heuristic by @alexpasmantier in [#58](https://github.com/alexpasmantier/television/pull/58)



### New Contributors
* @github-actions[bot] made their first contribution in [#73](https://github.com/alexpasmantier/television/pull/73)
* @MohamedBsh made their first contribution
* @bertrand-chardon made their first contribution in [#59](https://github.com/alexpasmantier/television/pull/59)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.5.1...0.5.3


## [0.5.1](https://github.com/alexpasmantier/television/releases/tag/0.5.1) - 2024-11-20

### 📚 Documentation

- [f43b5bf](https://github.com/alexpasmantier/television/commit/f43b5bf9b8fe034e958bec100f2d4569c87878be) *(brew)* Add brew installation method for MacOS to README by @alexpasmantier in [#45](https://github.com/alexpasmantier/television/pull/45)

- [30639c6](https://github.com/alexpasmantier/television/commit/30639c66b037733f6db0300b4573a1ccd2e33093) *(config)* Update docs to mention XDG_CONFIG_HOME precedence on all platform by @alexpasmantier in [#48](https://github.com/alexpasmantier/television/pull/48)

- [8a7b3da](https://github.com/alexpasmantier/television/commit/8a7b3da7fa20024bf5201c387260a36a16884b45) *(uncategorized)* Add instructions for installing on Arch Linux by @orhun in [#43](https://github.com/alexpasmantier/television/pull/43)

### ⚙️ Miscellaneous Tasks

- [9dcb223](https://github.com/alexpasmantier/television/commit/9dcb223dbac93b79f5913c782ab601446bab6052) *(actions)* Remove changelog update from the main branch by @alexpasmantier

- [6540094](https://github.com/alexpasmantier/television/commit/6540094cc9977419a92c4dcf37d761bebd5f052a) *(changelog)* Udpate changelog and add corresponding makefile command by @alexpasmantier in [#53](https://github.com/alexpasmantier/television/pull/53)

- [ccd7c68](https://github.com/alexpasmantier/television/commit/ccd7c687026ecca6f6d43b843a805089b5bfe4b1) *(config)* Default configuration now uses 100% of terminal screen space by @alexpasmantier in [#47](https://github.com/alexpasmantier/television/pull/47)

- [d3564f2](https://github.com/alexpasmantier/television/commit/d3564f2aca060838c5bbba01ad40427379e90060) *(uncategorized)* Bump version to 0.5.1 by @alexpasmantier

- [3bf04d7](https://github.com/alexpasmantier/television/commit/3bf04d77858f69f79c161c94dca7f52ca17ba50f) *(uncategorized)* Add CHANGELOG.md by @alexpasmantier in [#44](https://github.com/alexpasmantier/television/pull/44)



### New Contributors
* @fredmorcos made their first contribution in [#50](https://github.com/alexpasmantier/television/pull/50)
* @orhun made their first contribution in [#43](https://github.com/alexpasmantier/television/pull/43)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.5.0...0.5.1


## [0.5.0](https://github.com/alexpasmantier/television/releases/tag/0.5.0) - 2024-11-18

### ⛰️  Features

- [5807cda](https://github.com/alexpasmantier/television/commit/5807cda45d0f9935617c92e2b47a6d54712f93bc) *(cli)* Allow passing passthrough keybindings via stdout for the parent process to deal with by @alexpasmantier in [#39](https://github.com/alexpasmantier/television/pull/39)

- [40d5b20](https://github.com/alexpasmantier/television/commit/40d5b20c7d5fd6dd6b32a07f40eafb37d16b4cfd) *(ui)* Make the top UI help bar toggleable by @alexpasmantier in [#41](https://github.com/alexpasmantier/television/pull/41)

### 🚜 Refactor

- [75d0bf7](https://github.com/alexpasmantier/television/commit/75d0bf7b6b4c7139b5fd0862e595b63b93e322bb) *(config)* Make action names snake case in keybinding configuration by @alexpasmantier in [#40](https://github.com/alexpasmantier/television/pull/40) [**breaking**]

### 📚 Documentation

- [5c44432](https://github.com/alexpasmantier/television/commit/5c44432776cfd1bdaae2d9a82a7caba2af0b7ac9) *(uncategorized)* Update README television version by @alexpasmantier

- [cb7a245](https://github.com/alexpasmantier/television/commit/cb7a24537c3f1e85d8050a39ba0eae49e9f6db69) *(uncategorized)* Update README television version specifier by @alexpasmantier

- [da5c903](https://github.com/alexpasmantier/television/commit/da5c90317792f61abb0d793ed83b4d1728d2cb0e) *(uncategorized)* Update README television version by @alexpasmantier

### ⚙️ Miscellaneous Tasks

- [480059e](https://github.com/alexpasmantier/television/commit/480059eaaee16da11718ad765eda5e0c90cef4d7) *(rustfmt)* Update rustfmt.toml by @alexpasmantier in [#42](https://github.com/alexpasmantier/television/pull/42)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.4.23...0.5.0


## [0.4.23](https://github.com/alexpasmantier/television/releases/tag/0.4.23) - 2024-11-16

### ⛰️  Features

- [512afa2](https://github.com/alexpasmantier/television/commit/512afa2fda3a679ce0dc4ed37f85b177b3a215f6) *(ui)* Make help bar display optional by @alexpasmantier in [#35](https://github.com/alexpasmantier/television/pull/35)

### 🚜 Refactor

- [aa2f260](https://github.com/alexpasmantier/television/commit/aa2f2609a438768866d333713a938453eba1b402) *(configuration)* Modularize code and better handling of default options by @alexpasmantier in [#32](https://github.com/alexpasmantier/television/pull/32)

### 📚 Documentation

- [7277a3f](https://github.com/alexpasmantier/television/commit/7277a3f3ab32d61a41ec0d4f8dd083855527e0a5) *(config)* Update docs default configuration by @alexpasmantier in [#34](https://github.com/alexpasmantier/television/pull/34)

- [45e14d3](https://github.com/alexpasmantier/television/commit/45e14d3fa20a8e708fdc8ec75f74f34e8b86b0da) *(debian)* Add installation docs for debian-based systems by @alexpasmantier in [#33](https://github.com/alexpasmantier/television/pull/33)




**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.4.22...0.4.23


## [0.4.22](https://github.com/alexpasmantier/television/releases/tag/0.4.22) - 2024-11-16

### 🐛 Bug Fixes

- [06a4feb](https://github.com/alexpasmantier/television/commit/06a4feb9f2a1b191d7f1773d7fc99cb5565da407) *(config)* Swap out default keymaps with user defined ones instead of stacking by @alexpasmantier in [#26](https://github.com/alexpasmantier/television/pull/26)

- [f47b8be](https://github.com/alexpasmantier/television/commit/f47b8be9de8c1bfd29a08eea90e10c2d03865003) *(ghactions)* Only trigger cd workflow on new tags by @alexpasmantier in [#22](https://github.com/alexpasmantier/television/pull/22)

### 🚜 Refactor

- [4f0daec](https://github.com/alexpasmantier/television/commit/4f0daec63d868e16b1aa0349652ce9480623a496) *(channels)* Converting between entries and channels is now generic over channels by @alexpasmantier in [#25](https://github.com/alexpasmantier/television/pull/25)

### ⚙️ Miscellaneous Tasks

- [dcf9f6a](https://github.com/alexpasmantier/television/commit/dcf9f6a62156f425e378ac346ad6f18466076356) *(cd)* Fix cd configuration for deb packages by @alexpasmantier

- [e9dde70](https://github.com/alexpasmantier/television/commit/e9dde70ecf4bf48ae0f16c19f2b0aa296b6af777) *(cd)* Fix cd configuration for deb packages by @alexpasmantier

- [900bfa5](https://github.com/alexpasmantier/television/commit/900bfa50b92e2f023afc78fe4a4bed618480c2e5) *(deb)* Release deb package for television by @alexpasmantier

- [d0f023c](https://github.com/alexpasmantier/television/commit/d0f023cf1848055a7d83f6b81b286bd5e14237da) *(versions)* Bump workspace crates versions by @alexpasmantier

- [d50337b](https://github.com/alexpasmantier/television/commit/d50337b5c51c45f48a5a09431ff1b85c45964da2) *(uncategorized)* Update CD workflow by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/v0.4.21...0.4.22


## [v0.4.21](https://github.com/alexpasmantier/television/releases/tag/v0.4.21) - 2024-11-13

### 🐛 Bug Fixes

- [ff25fb2](https://github.com/alexpasmantier/television/commit/ff25fb2ddeb9c6f70294e5099a617219e30248d8) *(windows)* #20 respect `TELEVISION_CONFIG` env var on windows by @alexpasmantier in [#21](https://github.com/alexpasmantier/television/pull/21)

### ⚙️ Miscellaneous Tasks

- [65bb26e](https://github.com/alexpasmantier/television/commit/65bb26ec847e0d2caae49fbaeb3bffef90e094cd) *(nix)* Nix flake shell + rust-toolchain.toml setup by @tukanoidd in [#14](https://github.com/alexpasmantier/television/pull/14)



### New Contributors
* @tukanoidd made their first contribution in [#14](https://github.com/alexpasmantier/television/pull/14)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/v0.4.20...v0.4.21


## [v0.4.20](https://github.com/alexpasmantier/television/releases/tag/v0.4.20) - 2024-11-11

### 🐛 Bug Fixes

- [b1fe018](https://github.com/alexpasmantier/television/commit/b1fe0182f8f8de8ea5834fc3b148b53666d4349a) *(cargo workspace)* Fix cargo workspace structure and dependencies by @alexpasmantier in [#15](https://github.com/alexpasmantier/television/pull/15)

- [81cf17b](https://github.com/alexpasmantier/television/commit/81cf17bd5d883f581b5958ae70995a8acdd6e9d2) *(config)* More consistent configuration file location for linux and macos by @alexpasmantier in [#9](https://github.com/alexpasmantier/television/pull/9)

- [b3760d2](https://github.com/alexpasmantier/television/commit/b3760d2259951cc904f1fde7d7ac18d20f94b73c) *(windows)* Bump television_utils to v0.0.1 by @alexpasmantier in [#4](https://github.com/alexpasmantier/television/pull/4)

- [e475523](https://github.com/alexpasmantier/television/commit/e475523c797a46c7f229558789e8a1856c5adc23) *(windows)* Ignore `KeyEventKind::Release` events by @ErichDonGubler in [#3](https://github.com/alexpasmantier/television/pull/3)

- [d2e7789](https://github.com/alexpasmantier/television/commit/d2e7789612b22174e3ff24b0c7afe2da421cf5e7) *(workspace)* Fix cargo workspace dependencies by @alexpasmantier

### 🚜 Refactor

- [5611ee8](https://github.com/alexpasmantier/television/commit/5611ee8b2d7b02d9af311c31f6c2366dd2224248) *(workspace)* Reorganize cargo workspace by @alexpasmantier in [#12](https://github.com/alexpasmantier/television/pull/12)

### 📚 Documentation

- [cc9924d](https://github.com/alexpasmantier/television/commit/cc9924dd614b1b1625e019f76b8465e9b88880c3) *(readme)* Update terminal emulators compatibility list by @alexpasmantier in [#6](https://github.com/alexpasmantier/television/pull/6)

- [0c13626](https://github.com/alexpasmantier/television/commit/0c13626d4c1b1799ffc8e5f68b731222c3234dbd) *(uncategorized)* Fix table alignments by @alexpasmantier

- [6b0a038](https://github.com/alexpasmantier/television/commit/6b0a0387382f0d1bf61e2adbeca2276dd71b9836) *(uncategorized)* Add terminal emulators compatibility status by @alexpasmantier

### ⚡ Performance

- [62073d6](https://github.com/alexpasmantier/television/commit/62073d69ccc022d75bcc6bc5adc4472bdfe5b7f5) *(preview)* Remove temporary plaintext previews in favor of loading message preview by @alexpasmantier in [#10](https://github.com/alexpasmantier/television/pull/10)

### ⚙️ Miscellaneous Tasks

- [3a9ff06](https://github.com/alexpasmantier/television/commit/3a9ff067afad7e317fa5a34a95ba9ccbcca3e9ef) *(coc)* Create CODE_OF_CONDUCT.md by @alexpasmantier in [#7](https://github.com/alexpasmantier/television/pull/7)

- [7bc6f29](https://github.com/alexpasmantier/television/commit/7bc6f29c30334218da6baaeef1ddb02fdaa06a5c) *(crate)* Add include directives to Cargo.toml to make the crate leaner by @alexpasmantier in [#11](https://github.com/alexpasmantier/television/pull/11)

- [b8ad340](https://github.com/alexpasmantier/television/commit/b8ad34060d506c41a1ff491258edb09419b33178) *(uncategorized)* Update README.md install section by @alexpasmantier



### New Contributors
* @tranzystorekk made their first contribution in [#5](https://github.com/alexpasmantier/television/pull/5)
* @ErichDonGubler made their first contribution in [#3](https://github.com/alexpasmantier/television/pull/3)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/v0.4.18...v0.4.20


## [v0.4.18](https://github.com/alexpasmantier/television/releases/tag/v0.4.18) - 2024-11-10

### 🐛 Bug Fixes

- [c70e675](https://github.com/alexpasmantier/television/commit/c70e6756553bbeb1bc9332a7b011fddf24be52c0) *(uncategorized)* Add `winapi-util` dependency for windows builds by @alexpasmantier

- [df7020a](https://github.com/alexpasmantier/television/commit/df7020a7a82e82cace2fa84d24182c7a0911613d) *(uncategorized)* Add the correct permissions to release binaries by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/v0.4.17...v0.4.18


## [v0.4.17](https://github.com/alexpasmantier/television/releases/tag/v0.4.17) - 2024-11-10

### ⚙️ Miscellaneous Tasks

- [2f5640f](https://github.com/alexpasmantier/television/commit/2f5640f4cde0a61d6dc9946c8b73bc3c2b54e4dd) *(uncategorized)* Testing out the CD pipeline by @alexpasmantier

- [2e49862](https://github.com/alexpasmantier/television/commit/2e49862a7e40b87b704eaf3ef0a30b8cf483cb24) *(uncategorized)* Update Makefile and CONTRIBUTING.md by @alexpasmantier

- [6eafb7b](https://github.com/alexpasmantier/television/commit/6eafb7bfe800e0a96d52674a46903e06238536d0) *(uncategorized)* Udate documentation and dependencies by @alexpasmantier




**Full Changelog**: https://github.com/alexpasmantier/television/compare/v0.4.15...v0.4.17


## [v0.4.15](https://github.com/alexpasmantier/television/releases/tag/v0.4.15) - 2024-11-10

### ⛰️  Features

- [759815a](https://github.com/alexpasmantier/television/commit/759815ae24dd471365455b932922fb66773eb50b) *(uncategorized)* More syntaxes and themes for highlighting + configuration by @alexpasmantier

- [d0d453f](https://github.com/alexpasmantier/television/commit/d0d453fe9748c42b7d81d7a2bfbad6fe0d966c84) *(uncategorized)* Send to channel by @alexpasmantier

### 🐛 Bug Fixes

- [32c114a](https://github.com/alexpasmantier/television/commit/32c114aa9fa51c1f74b15b6d38ba904f9cfce557) *(uncategorized)* Gag stdout and stderr while loading theme assets to silence bat warning by @alexpasmantier

- [f449477](https://github.com/alexpasmantier/television/commit/f449477605bb48f6c18334440dbc9d360b0ec43e) *(uncategorized)* Doctests imports by @alexpasmantier

- [de74b61](https://github.com/alexpasmantier/television/commit/de74b619b86b81feb165c5518995d36ca9a0bada) *(uncategorized)* Stabilize preview scroll initialization by @alexpasmantier

- [dd14bd4](https://github.com/alexpasmantier/television/commit/dd14bd4f8d2ff58aed9bfda2ca6fc8c0f9a74729) *(uncategorized)* Filtering system directories in gitrepos by @alexpasmantier

### 🚜 Refactor

- [8dd7f23](https://github.com/alexpasmantier/television/commit/8dd7f237345601a976c55b112d71e493bf83d2e2) *(uncategorized)* More refactoring and fixing doctests by @alexpasmantier

- [ae938dc](https://github.com/alexpasmantier/television/commit/ae938dcfc0778ef85df3b8f81cd35edec737f644) *(uncategorized)* Split project into separate crates by @alexpasmantier

- [c1f41bf](https://github.com/alexpasmantier/television/commit/c1f41bf107e5352ac910543cd1b447193af494cd) *(uncategorized)* Extract matcher logic into separate crate by @alexpasmantier

### 📚 Documentation

- [cd31619](https://github.com/alexpasmantier/television/commit/cd31619c8ab7df6975f6d26d9948617318d05de0) *(readme)* Update README.md by @alexpasmantier

- [51a98db](https://github.com/alexpasmantier/television/commit/51a98db9d564f02e0ef9b3bc3242439ea74c7406) *(readme)* Update README.md by @alexpasmantier

- [c7fbe26](https://github.com/alexpasmantier/television/commit/c7fbe26596561e5155d5a52f04957fbcb168397f) *(readme)* Update README.md by @alexpasmantier

- [ef4ab70](https://github.com/alexpasmantier/television/commit/ef4ab705b44d0b4644e859c13bb804815226259f) *(readme)* Update README.md by @alexpasmantier

- [068ed88](https://github.com/alexpasmantier/television/commit/068ed8813c5bd51aea290842667eb25cfd26d7b9) *(readme)* Update README.md by @alexpasmantier

- [cfa4178](https://github.com/alexpasmantier/television/commit/cfa41789bc850a3078e97278878336985f487b08) *(readme)* Update README.md by @alexpasmantier

- [37fb013](https://github.com/alexpasmantier/television/commit/37fb013f0cdaf9d97ea84f4432f8348b18bbc340) *(uncategorized)* More work on CONTRIBUTING.md by @alexpasmantier

- [b0ab8a1](https://github.com/alexpasmantier/television/commit/b0ab8a179aa72dbd42c8928d2425bd0d9d7ef22f) *(uncategorized)* Some work on CONTRIBUTING.md by @alexpasmantier

- [19f00f5](https://github.com/alexpasmantier/television/commit/19f00f5916e1f3a2a4d2320c84eb2c1ea2858a8b) *(uncategorized)* Add default keybindings to README.md by @alexpasmantier

- [96976d9](https://github.com/alexpasmantier/television/commit/96976d93cb4a7859c25599269f6ba87229afecfe) *(uncategorized)* Update README.md by @alexpasmantier

### ⚙️ Miscellaneous Tasks

- [4e4ef97](https://github.com/alexpasmantier/television/commit/4e4ef9761b997badd5a57347d62f9c3e617deff8) *(precommit)* Don't allow committing if clippy doesn't pass by @alexpasmantier

- [b04e182](https://github.com/alexpasmantier/television/commit/b04e1824535467f401d7117b0e6048b2dfabb7fe) *(previewers)* Unused attributes by @alexpasmantier

- [d2005e1](https://github.com/alexpasmantier/television/commit/d2005e1116b7830ee3d85c0fc7dec35ac4e5e99d) *(uncategorized)* Bump version by @alexpasmantier

- [79da161](https://github.com/alexpasmantier/television/commit/79da161943c0cd2865c5931b8c251417035c393d) *(uncategorized)* Add license to syntax snippet by @alexpasmantier

- [5b57d6b](https://github.com/alexpasmantier/television/commit/5b57d6b29019a67706ee354d32b23ebbadb710ba) *(uncategorized)* Update workspace crates configurations by @alexpasmantier

- [c4863ff](https://github.com/alexpasmantier/television/commit/c4863ff7ae55fd1536caf7a490deb21bf9be7329) *(uncategorized)* Patch by @alexpasmantier

- [9bdbf44](https://github.com/alexpasmantier/television/commit/9bdbf44f35e92740e7b0ac4e8c26d299ca6fa1ef) *(uncategorized)* Makefile and dist scripts by @alexpasmantier

- [b913eac](https://github.com/alexpasmantier/television/commit/b913eac4ae0f3767d1495c95902ce8be0d33656d) *(uncategorized)* Update dependencies and bump version by @alexpasmantier

- [2dbbd0c](https://github.com/alexpasmantier/television/commit/2dbbd0c4a3b227062402d7c994b4dc6b3a8eeb87) *(uncategorized)* Bump version by @alexpasmantier

- [8fe1246](https://github.com/alexpasmantier/television/commit/8fe1246923939f16536aa276ca5a3b878982001d) *(uncategorized)* Update dependencies and bump version by @alexpasmantier

- [3d647b2](https://github.com/alexpasmantier/television/commit/3d647b20103b3609a7d4edb372b24341fa0d03dc) *(uncategorized)* Update dependencies and bump version by @alexpasmantier

- [7b18c4f](https://github.com/alexpasmantier/television/commit/7b18c4f88d562e9a1a32d4685fa4d039363c6f3c) *(uncategorized)* Unused imports and ci docs by @alexpasmantier

- [e83fabb](https://github.com/alexpasmantier/television/commit/e83fabbc0b6e691a40eff4ffc190dc94516b3841) *(uncategorized)* Bump version by @alexpasmantier

- [dbc4b6c](https://github.com/alexpasmantier/television/commit/dbc4b6c06a57bcc6528bfa180de495a444588515) *(uncategorized)* Bump version by @alexpasmantier



### New Contributors
* @alexpasmantier made their first contribution


<!-- generated by git-cliff -->

