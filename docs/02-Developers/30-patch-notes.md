# Release notes for television 0.12.0

![image](https://github.com/user-attachments/assets/96a904db-7b02-4457-80e2-6c00b5356769)

## New Contributors

- @Ktoks made their first contribution in [#590](https://github.com/alexpasmantier/television/pull/590)
- @lalvarezt made their first contribution in [#588](https://github.com/alexpasmantier/television/pull/588)
- @kapobajza made their first contribution in [#568](https://github.com/alexpasmantier/television/pull/568)
- @domaschh made their first contribution
- @cr4ftx made their first contribution in [#496](https://github.com/alexpasmantier/television/pull/496)

## Highlights

This section is meant as a quick recap of what you should know when upgrading to 0.12.0.

It is in no means exhaustive. If you're really interested in the complete changelog, feel free to [skip ahead](#changelog).

### Channels refactor and lots of new features

- channels now allow [much more configuration options](https://github.com/alexpasmantier/television/blob/main/docs/channels.md#channel-specification)
- channels are now laid out in [a more natural way](https://github.com/alexpasmantier/television/blob/main/docs/channels.md#default-location-on-your-system) in the user's config directory
- tv now relies on [string-pipeline](https://github.com/lalvarezt/string_pipeline) as its templating system which provides a concise and very expressive syntax that supports quite a lot of basic transformations
- channels can be accessed directly using keyboard shortcuts (see config options above)
- community-maintained channels on the official repo can now be installed directly via the cli
- channels can now be live reloaded and can be configured to live reload periodically

### CLI

Lots of new configuration options, all listed [here](https://github.com/alexpasmantier/television/blob/main/docs/advanced/cli.md#-source-and-data-options), among which:

- UI elements, sizes, layout (most of which are covered [here](https://github.com/alexpasmantier/television/blob/main/docs/advanced/cli.md#-source-and-data-options))
- channels can now be built on the fly using the cli:

```sh
tv --source-command "find . -name '*.rs'" \
    --preview-command "bat -n --color=always '{}'" \
    --preview-size 70
```

- you may now choose to disable some of tv's features for a specific use case (e.g. opening tv in single-channel mode by disabling the remote control entirely, or choosing to disable the status bar for an extra line of space, etc.)
- you may now define custom keybindings through the cli
- you may tweak tv's selection behavior using `--select-1`, `--take-1`, `--take-1-fast`
- tv now has a `--watch` mode
- you may now download channels from tv's repo using the cli

### New UI features and improvements to customization

![tv-files-remote](https://github.com/user-attachments/assets/625081f0-d8bf-45c2-9920-1d7e1f66918e)

- tv now has a status bar and a help panel
- the remote control went through a rework and now displays richer information about available channels
- tv now has a portrait mode  
  ![image](https://github.com/user-attachments/assets/e97aa7c2-e9cd-4eed-9d74-04ee3a35f0c5)
- the preview panel size is now configurable on a per channel basis
- tv now has an `--inline` mode (+ `--height`, `--width`)  
  ![image](https://github.com/user-attachments/assets/aeb8bb07-741a-42fa-8890-ae585e9194b2)
- more customizable UI elements
  ![image](https://github.com/user-attachments/assets/154bee6e-7e9f-4c43-97ed-0b0b372a620b)
- preview scrollbars

### Shell integration

- added support for nushell
- shell integration now spawns tv in `inline` mode by default
- improved shell integration for zsh and fish

### Others

- mouse support
- tui testing framework
- search history (per-channel and global)
- a lot of bug fixes
- substantial performance improvements while drawing much less resources
- heavy refactoring and simplifying the code (deleting nearly 10k loc)
- a fair amount of documentation work
- a lot of new tests

### New website

Television now has [a brand new website](https://alexpasmantier.github.io/television)!
![image](https://github.com/user-attachments/assets/d247c266-8525-454a-868c-638442278186)

## Changelog

### ⛰️ Features

- [c34fa57](https://github.com/alexpasmantier/television/commit/c34fa5731213afc1bd890b4cb76e5a9f6c689095) _(binary)_ Host our own apt repo by @kapobajza in [#568](https://github.com/alexpasmantier/television/pull/568)

- [7b40e76](https://github.com/alexpasmantier/television/commit/7b40e769aebf55000daa6437fcc27774ceb5c70b) _(cable)_ Migrate windows channels by @alexpasmantier

- [6b38ce2](https://github.com/alexpasmantier/television/commit/6b38ce2058ca14265ad2a4c93ee3a2603944987f) _(cable)_ Migrate the rest of unix channels by @alexpasmantier

- [a49f104](https://github.com/alexpasmantier/television/commit/a49f1046709d911761761f373df4e32179950341) _(channel)_ Add channel global shortcuts by @lalvarezt

- [1891736](https://github.com/alexpasmantier/television/commit/18917362beb039ef5336b8b60b977e2d608b7d8f) _(cli)_ Add watch flag to trigger reload of channels by @lalvarezt

- [2ecbc8a](https://github.com/alexpasmantier/television/commit/2ecbc8a170693bb68f3f517dcf95c6b690c1771f) _(cli)_ Initial support for source and preview overrides, layout, take_1 and take_1_fast by @lalvarezt

- [bc8d636](https://github.com/alexpasmantier/television/commit/bc8d6360056d73eb868d54272f703436dfe6ca15) _(cli)_ Add cli options to override configuration and cable directories by @alexpasmantier

- [f887a23](https://github.com/alexpasmantier/television/commit/f887a2390ede0a5f30d61f2bb9d4e1e421109d63) _(cli)_ Add a `--ui-scale` [0,100] cli parameter by @alexpasmantier in [#492](https://github.com/alexpasmantier/television/pull/492)

- [7067a2b](https://github.com/alexpasmantier/television/commit/7067a2ba939bba803d4a6d54423ba87476e40dc4) _(remote)_ Rework remote UI and add description and requirements panels by @alexpasmantier

- [cfe49ce](https://github.com/alexpasmantier/television/commit/cfe49ce81c1eb428b7c38fe5b524d67141099946) _(remote)_ Redirect `Action::Quit` to `Action::ToggleRemoteControl` when in remote mode by @alexpasmantier in [#508](https://github.com/alexpasmantier/television/pull/508)

- [4d80e95](https://github.com/alexpasmantier/television/commit/4d80e95c6b42bd3d5b9d891e780df66b5a6235cc) _(shell)_ Add support for integration with NuShell by @alexpasmantier in [#410](https://github.com/alexpasmantier/television/pull/410)

- [0f4d879](https://github.com/alexpasmantier/television/commit/0f4d87915b76c066a9a4f9ac01f81763271ce37e) _(shell)_ Improve zsh completion system by @lalvarezt in [#525](https://github.com/alexpasmantier/television/pull/525)

- [be8008e](https://github.com/alexpasmantier/television/commit/be8008e97d5ab5063aff27bea52b6315b9f878f7) _(shell)_ Improve fish completion system by @lalvarezt in [#494](https://github.com/alexpasmantier/television/pull/494)

- [639caa1](https://github.com/alexpasmantier/television/commit/639caa1a30cf1a9df78e2462e8cce98cf4c53d16) _(stdin)_ Accept various entry separator characters by @alexpasmantier in [#572](https://github.com/alexpasmantier/television/pull/572)

- [ccc12e2](https://github.com/alexpasmantier/television/commit/ccc12e2644aa329589ca55c112a131fc16163a86) _(tui)_ Add special testing conditions for overlay testing by @lalvarezt in [#585](https://github.com/alexpasmantier/television/pull/585)

- [4ed48cc](https://github.com/alexpasmantier/television/commit/4ed48ccdadf05f4b4ca09aeea8eaf82c6d798486) _(ui)_ Support for non-fullscreen UI by @lalvarezt in [#578](https://github.com/alexpasmantier/television/pull/578)

- [23f52d4](https://github.com/alexpasmantier/television/commit/23f52d4533d55223f2b60d431d9a7915409795ef) _(ui)_ Optional scrollbar and mouse support for the preview panel by @lalvarezt

- [ad4e254](https://github.com/alexpasmantier/television/commit/ad4e254ae652e9e2706a81abe01e9ae6e1b2dc51) _(ui)_ New keybindings panel and status bar by @lalvarezt

- [510e7b6](https://github.com/alexpasmantier/television/commit/510e7b633829e34adb21ade72163993fd11b3c15) _(ui)_ Add support for customizing `input_header`, `preview_header` and `preview_footer` by @lalvarezt

- [783d96b](https://github.com/alexpasmantier/television/commit/783d96bb678de59a10fff63a8719efe4dd5e50fc) _(ui)_ Preview size customization by @lalvarezt

- [1086899](https://github.com/alexpasmantier/television/commit/1086899ba76f9b3377a4f67d8d7aef5da2cd310d) _(ui)_ Add a UI portrait mode #489 by @cr4ftx in [#496](https://github.com/alexpasmantier/television/pull/496)

- [3b3a0ec](https://github.com/alexpasmantier/television/commit/3b3a0ec1ffc8a2ccd9b7f2dd890d752933c8ff31) _(windows)_ Add text channel with preview offset for windows by @alexpasmantier in [#514](https://github.com/alexpasmantier/television/pull/514)

- [4513945](https://github.com/alexpasmantier/television/commit/45139457a15773b10c6b1963f02947c2738d7eed) _(uncategorized)_ Add global/channel input history by @lalvarezt in [#573](https://github.com/alexpasmantier/television/pull/573)

- [9e306d9](https://github.com/alexpasmantier/television/commit/9e306d93bc3e95441c042452d503770d8af0c6e4) _(uncategorized)_ New channel and added reload and toggle actions by @lalvarezt

### 🐛 Bug Fixes

- [dbff3a3](https://github.com/alexpasmantier/television/commit/dbff3a330b169c422ae384e373b934dceb8e01b2) _(alias)_ Move terminal raw mode before loading bat assets #444 by @cr4ftx in [#484](https://github.com/alexpasmantier/television/pull/484)

- [0514a91](https://github.com/alexpasmantier/television/commit/0514a914b630719391d66df61eb9d53c58933c3f) _(alias)_ Rename the aliases channel to `alias` by @alexpasmantier in [#485](https://github.com/alexpasmantier/television/pull/485)

- [67195e7](https://github.com/alexpasmantier/television/commit/67195e756c85514c144232800db21b49d8aa0de1) _(app)_ Channel keybindings are ignored by @lalvarezt

- [415dd38](https://github.com/alexpasmantier/television/commit/415dd38c66b93b96bdc6d1701494c1fbb068a78d) _(app)_ Honor cli no-help and no-preview by @lalvarezt

- [6b3c4ee](https://github.com/alexpasmantier/television/commit/6b3c4ee773fb53cd72e384e892faf29d83fd63c7) _(cable)_ Don't panic when unable to format user template with entry by @alexpasmantier in [#516](https://github.com/alexpasmantier/television/pull/516)

- [5d730cd](https://github.com/alexpasmantier/television/commit/5d730cdf71141ea8224e2a7d617a5431a705aaf6) _(channel)_ Only allow reload and cycle_sources in channel mode by @alexpasmantier

- [17439da](https://github.com/alexpasmantier/television/commit/17439dace5c5bfad49fd7e4c1344e520a1fd3c6b) _(channels)_ Quote bat arguments by @Ktoks in [#590](https://github.com/alexpasmantier/television/pull/590)

- [07556ea](https://github.com/alexpasmantier/television/commit/07556eac79f08c74472e3da276df3be2d71e85b1) _(cli)_ Fix validation rules when reading from stdin by @alexpasmantier

- [ca5808a](https://github.com/alexpasmantier/television/commit/ca5808affe9166babea5f194d6ccc58daef37f38) _(cli)_ Fix parsing of arguments for autocomplete-prompt by @lalvarezt in [#569](https://github.com/alexpasmantier/television/pull/569)

- [090d71a](https://github.com/alexpasmantier/television/commit/090d71aff0112b22631764ffae5b73d2bb8a87c5) _(cli)_ Using --exact now works on the --input text aswell by @domaschh

- [dfbdd65](https://github.com/alexpasmantier/television/commit/dfbdd65107ebd189fc3ebaa9b1650d012391aeee) _(config)_ Use the config `default_channel` field as a fallback when no channel is specified by @alexpasmantier in [#524](https://github.com/alexpasmantier/television/pull/524)

- [653c986](https://github.com/alexpasmantier/television/commit/653c986b7ee9d5bcb130b50483a83be3ca48e0e1) _(github)_ Copy github prototypes' content directly by @alexpasmantier

- [3e98475](https://github.com/alexpasmantier/television/commit/3e98475e3529e0cf63bf1e5f1667888e4e0d41cd) _(github)_ Improve ux when downloading cable channels by @alexpasmantier

- [9a80919](https://github.com/alexpasmantier/television/commit/9a80919f66c576177c76f6468eb462ee746dbc0f) _(keybindings)_ Add cmd as an option for modifrs by @domaschh

- [cd33151](https://github.com/alexpasmantier/television/commit/cd33151bac9422dcef8edcfd16a6553228611631) _(layout)_ Double check whether preview is enabled by @nkxxll in [#499](https://github.com/alexpasmantier/television/pull/499)

- [d429a9a](https://github.com/alexpasmantier/television/commit/d429a9a6ee97022d21d7f914a4288efe291a1cc1) _(matcher)_ Better handling of reloading and cycling through sources by @alexpasmantier

- [94e34c1](https://github.com/alexpasmantier/television/commit/94e34c16682e2af56a60511dce5c44e26e8aa914) _(os)_ No more panicking on cwd-related errors by @alexpasmantier

- [0f8a585](https://github.com/alexpasmantier/television/commit/0f8a585c85befebcdbcbba1b12ca774c3f973b64) _(preview)_ Default to no offset when offset template formatting fails by @alexpasmantier

- [a81a86f](https://github.com/alexpasmantier/television/commit/a81a86f1fd01e049b33a6afdb58adfcadaad1095) _(preview)_ Don't panic when the previewer attempts to send to a closed channel by @alexpasmantier

- [1741a15](https://github.com/alexpasmantier/television/commit/1741a15e526ea0a304bb1cccb5f75bb46d42a6a2) _(preview)_ Add a post-processing step to clean out ansi text from non-displayable characters by @alexpasmantier in [#509](https://github.com/alexpasmantier/television/pull/509)

- [a8fb0f0](https://github.com/alexpasmantier/television/commit/a8fb0f0b0e771ae3574b71630ff43e15b7ffc9ef) _(reload)_ Avoid UI flickering while reloading channel by @alexpasmantier

- [1f0c178](https://github.com/alexpasmantier/television/commit/1f0c178a2d79ccf1e6cbe13ea3ec246f987bfbf2) _(results)_ Remove keymap hint if help is disabled by @nkxxll in [#480](https://github.com/alexpasmantier/television/pull/480)

- [39dd9ef](https://github.com/alexpasmantier/television/commit/39dd9efd5dfa1fb36281f9f97b753152af82095f) _(shell)_ Paste not working in zsh shell integration by @kapobajza in [#512](https://github.com/alexpasmantier/television/pull/512)

- [1de2866](https://github.com/alexpasmantier/television/commit/1de28665d90af0a566a2ab16c92194738faa04d7) _(templates)_ Handle case when template contains brackets that shouldn't be interpreted by the parser by @alexpasmantier

- [dde3193](https://github.com/alexpasmantier/television/commit/dde319359fba900f03deae566bfbb17633a2c081) _(tui)_ Fixed shell completion widget not rendering, add poc for fish by @lalvarezt in [#588](https://github.com/alexpasmantier/television/pull/588)

- [dc75e80](https://github.com/alexpasmantier/television/commit/dc75e80fb93223d3e9992ff21ac67b5ff28987fa) _(ui)_ Avoid glitches caused by programs outputting control sequences by @alexpasmantier in [#579](https://github.com/alexpasmantier/television/pull/579)

- [e5a13ef](https://github.com/alexpasmantier/television/commit/e5a13ef8a12823e8369af9bce68fe18749e7b1ec) _(uncategorized)_ Reset picker selection when cycling through sources by @alexpasmantier

- [b0c25b1](https://github.com/alexpasmantier/television/commit/b0c25b19be2437ef250c4064906e006d55343816) _(uncategorized)_ Rollback unwanted modifications on text and env prototypes by @alexpasmantier

- [175015d](https://github.com/alexpasmantier/television/commit/175015d470f3e3bc7e78fe41a56f9be87123c9b4) _(uncategorized)_ Load new channel after zapping with the remote by @alexpasmantier

- [c80e9b1](https://github.com/alexpasmantier/television/commit/c80e9b18cb39d97927c93317b599ba12d4d80cef) _(uncategorized)_ "toggle source" leftovers by @alexpasmantier

### 🚜 Refactor

- [2fdb47f](https://github.com/alexpasmantier/television/commit/2fdb47fc196347e9076da48bb873e9968ad4e0c4) _(cable)_ Add helper function to get keybindings by @lalvarezt

- [e06e5e6](https://github.com/alexpasmantier/television/commit/e06e5e6a379f52580126e9bbbd8d0722a6168fc3) _(cable)_ Update preview size for `files` and `env` by @alexpasmantier

- [e76a3df](https://github.com/alexpasmantier/television/commit/e76a3df776ffd71b12eadee9bf82bd2abda0e553) _(cable)_ Cable format redesign by @alexpasmantier [**breaking**]

- [b372fe8](https://github.com/alexpasmantier/television/commit/b372fe86ea5532e5e0c400d9f45e1517b95595ad) _(cable)_ Add `files` and `text` channels with the new configuration format by @alexpasmantier in [#534](https://github.com/alexpasmantier/television/pull/534)

- [e2f52b8](https://github.com/alexpasmantier/television/commit/e2f52b835d6447c251d7fca6724cf409ed153546) _(cable)_ Improve naming and documentation for `prototypes.rs` by @alexpasmantier in [#487](https://github.com/alexpasmantier/television/pull/487)

- [4385317](https://github.com/alexpasmantier/television/commit/4385317e069db287d8d86f987e11e079a7ff6d1c) _(cable)_ Split cable related code into separate submodules by @alexpasmantier in [#486](https://github.com/alexpasmantier/television/pull/486)

- [1a5fa5d](https://github.com/alexpasmantier/television/commit/1a5fa5dd4cb485e2b0b08301ca457fa1c6d06094) _(channels)_ Some renaming and refactoring the channels module by @alexpasmantier in [#503](https://github.com/alexpasmantier/television/pull/503)

- [a2ebbb3](https://github.com/alexpasmantier/television/commit/a2ebbb35573dff9d06954962f8e5a58b84ab84cc) _(cli)_ Add validation logic + docs by @lalvarezt

- [ebca4d2](https://github.com/alexpasmantier/television/commit/ebca4d2017bd8298e2d50df3d1fbcfd0e56805c1) _(cli)_ Move cli overrides to dedicated function by @alexpasmantier

- [80cb6c3](https://github.com/alexpasmantier/television/commit/80cb6c3606481bfad26a4ad453848d5b9d25785f) _(picker)_ New movement system by @lalvarezt

- [b9f42e8](https://github.com/alexpasmantier/television/commit/b9f42e8c29a7eca86a91a6cb00d9c4ee46bb2bd3) _(preview)_ Simplify channel previews code and remove intermediate `PreviewKind` struct by @alexpasmantier in [#490](https://github.com/alexpasmantier/television/pull/490)

- [67c067f](https://github.com/alexpasmantier/television/commit/67c067ff40f97eef9090c2a5addca5da50a7fa0f) _(previewer)_ A much more efficient preview system for tv by @alexpasmantier in [#506](https://github.com/alexpasmantier/television/pull/506)

- [f138e8a](https://github.com/alexpasmantier/television/commit/f138e8a591ca4d3ff173ec316ce59b02fb5aca47) _(screen)_ Remove leftover line number, not used anymore by @lalvarezt

- [7ac2f28](https://github.com/alexpasmantier/television/commit/7ac2f28be7c475fb59b1388e443f0f33898ce0b6) _(screen)_ New result line abstraction that can be reused by @lalvarezt

- [4b338f5](https://github.com/alexpasmantier/television/commit/4b338f524284dcbff547776af19611a5ca58b930) _(shell)_ Make use the new Binding system by @lalvarezt

- [58d90c6](https://github.com/alexpasmantier/television/commit/58d90c6d03c237b5b4f45cc04ef55b7b081c4638) _(ui)_ Feature based UI by @lalvarezt

- [8fd9163](https://github.com/alexpasmantier/television/commit/8fd91633e271099d83694ccbce26101da2faabcf) _(uncategorized)_ More stable behavior for `--inline`, `--width` and `--height` by @alexpasmantier in [#589](https://github.com/alexpasmantier/television/pull/589)

- [d82bf72](https://github.com/alexpasmantier/television/commit/d82bf7226b853c65638a42298cc31f773631d40e) _(uncategorized)_ Extract overlay tui logic into separate function and call it on resize events by @alexpasmantier

- [4faab40](https://github.com/alexpasmantier/television/commit/4faab403d22e4dc7e745c1d40d266704719ad2c3) _(uncategorized)_ QOL improvements on channels and CLI override logic by @lalvarezt in [#584](https://github.com/alexpasmantier/television/pull/584)

- [292c521](https://github.com/alexpasmantier/television/commit/292c5212051f9ecf212b248dc7914fe107986042) _(uncategorized)_ Use super for linux and windows and cmd for macos by @alexpasmantier in [#571](https://github.com/alexpasmantier/television/pull/571)

- [51617b1](https://github.com/alexpasmantier/television/commit/51617b1775d56180f9cd09ceef7777447d414c14) _(uncategorized)_ Clearer separation of channels vs remote, better deserialization of prototype sub-structures, etc. by @alexpasmantier

- [53c36f0](https://github.com/alexpasmantier/television/commit/53c36f040c8ab0ef4a2da88aa4b8d4c07568e2a7) _(uncategorized)_ Add reload and cycle source to default keymaps + default keymap changes by @alexpasmantier

- [466a743](https://github.com/alexpasmantier/television/commit/466a74341d7f64cc0f1164a1878467c760277012) _(uncategorized)_ Strip new channels to their bare minimum config by @alexpasmantier

- [2b2654b](https://github.com/alexpasmantier/television/commit/2b2654b6aab86707577c0bb5c65301106422e737) _(uncategorized)_ Drop TelevisionChannel enum and all associated macros by @alexpasmantier in [#498](https://github.com/alexpasmantier/television/pull/498)

- [cc27b5e](https://github.com/alexpasmantier/television/commit/cc27b5ec6bf3a5a71d6785558e57976db9f2d129) _(uncategorized)_ Drop dependency to the `ignore` crate by @alexpasmantier

- [c2f4cc2](https://github.com/alexpasmantier/television/commit/c2f4cc258f5f3b21601e8c7ce98f4584222813b2) _(uncategorized)_ Tv no longer needs to write the default cable channel recipes to the user's configuration directory by @alexpasmantier in [#482](https://github.com/alexpasmantier/television/pull/482)

- [67677fb](https://github.com/alexpasmantier/television/commit/67677fb917b6d59d8217eaf6369b95f5ba940ff0) _(uncategorized)_ All channels are now cable channels by @alexpasmantier in [#479](https://github.com/alexpasmantier/television/pull/479) [**breaking**]

### 📚 Documentation

- [825e974](https://github.com/alexpasmantier/television/commit/825e97436360f3cbb5b40d4053c10c435352e6c9) _(cable)_ Update channel documentation (auto) by @github-actions[bot] in [#594](https://github.com/alexpasmantier/television/pull/594)

- [185f789](https://github.com/alexpasmantier/television/commit/185f7890ac27ca1580231c2f24fe6a696fd3f322) _(cable)_ Update channel documentation (auto) by @github-actions[bot]

- [8bdebd3](https://github.com/alexpasmantier/television/commit/8bdebd382ef9fe1f8ac74106b91cbdfdd308ef27) _(channels)_ Autogenerate channel documentation + CI action by @alexpasmantier

- [6015809](https://github.com/alexpasmantier/television/commit/601580953a11b4c1061c97c5417ffeacd154354d) _(tests)_ Simple documentation for the `PtyTester` by @alexpasmantier

- [7bbf538](https://github.com/alexpasmantier/television/commit/7bbf5388984ea4a9afa4daad695add4c258c0fb1) _(utils)_ Add documentation for string formatting logic by @alexpasmantier in [#517](https://github.com/alexpasmantier/television/pull/517)

- [0112b40](https://github.com/alexpasmantier/television/commit/0112b40df3d12a2f540ee7bbd5d6e24da6c2d048) _(uncategorized)_ Update installation docs by @alexpasmantier

- [e5654fc](https://github.com/alexpasmantier/television/commit/e5654fcddee638905a5e6478763e6b738ec469d1) _(uncategorized)_ Add docusaurus website by @alexpasmantier

- [e797aba](https://github.com/alexpasmantier/television/commit/e797aba7c1e5fbdc6afabac69d2bbcb32767bf80) _(uncategorized)_ Reorganize docs by @alexpasmantier

- [106004d](https://github.com/alexpasmantier/television/commit/106004de948327f248c14f8eebdc40b54af58a4c) _(uncategorized)_ Architecture by @alexpasmantier

- [1d33c93](https://github.com/alexpasmantier/television/commit/1d33c9391039f703b3b9848784ab58d3f5372a7d) _(uncategorized)_ Add readme banner by @alexpasmantier

- [3ac6c76](https://github.com/alexpasmantier/television/commit/3ac6c762335dd239a72556e36368858e97a13691) _(uncategorized)_ Update readme by @alexpasmantier

- [1c5810f](https://github.com/alexpasmantier/television/commit/1c5810fe75d3a049c9387bac6503b2be863a100a) _(uncategorized)_ Add assets + update readme image by @alexpasmantier

- [d9d6554](https://github.com/alexpasmantier/television/commit/d9d6554697275208bd75ecbcba256d591c828e36) _(uncategorized)_ Migrate some of the wiki's content to the docs/ folder by @alexpasmantier

- [9364b3e](https://github.com/alexpasmantier/television/commit/9364b3eb1561af8725e45bc36b01c505951ee7f4) _(uncategorized)_ Some cleaning and reorganizing by @alexpasmantier

- [f52d4ef](https://github.com/alexpasmantier/television/commit/f52d4ef524631b6b9a77a525d64f4a1094bf1857) _(uncategorized)_ Update README by @alexpasmantier

- [c25a5bd](https://github.com/alexpasmantier/television/commit/c25a5bd75f311a1fbe8b11d20f8022678042c755) _(uncategorized)_ Update readme and add new format docs by @alexpasmantier

- [aac7e4d](https://github.com/alexpasmantier/television/commit/aac7e4dc4570d5e0caf305b738009f2b077da7be) _(uncategorized)_ Update terminal emulators compatibility list by @alexpasmantier

- [9127e41](https://github.com/alexpasmantier/television/commit/9127e419fb4628dc3e65ee669315038a169bb8fe) _(uncategorized)_ Add index.md by @alexpasmantier

- [d3bb3b0](https://github.com/alexpasmantier/television/commit/d3bb3b0a5610b6896a698f89afcf2fb7a2aab44a) _(uncategorized)_ Cleanup old todo list by @alexpasmantier in [#483](https://github.com/alexpasmantier/television/pull/483)

### ⚡ Performance

- [fc2f8b9](https://github.com/alexpasmantier/television/commit/fc2f8b9473d1d84712951184da8d4e59edeedc86) _(previews)_ Avoid unnecessary preview content copy by @alexpasmantier in [#507](https://github.com/alexpasmantier/television/pull/507)

- [14804f5](https://github.com/alexpasmantier/television/commit/14804f50a27fa688ebed2afcacb96dd0453e89dc) _(uncategorized)_ Pre-allocate result vectors, and reuse picker entries to avoid reallocations by @lalvarezt

- [19d1ca1](https://github.com/alexpasmantier/television/commit/19d1ca155813a966c0ddc8650e664ab4030d6275) _(uncategorized)_ More pre-allocations and avoid unnecessary ascii string parsing work by @lalvarezt

### 🧪 Testing

- [f60b492](https://github.com/alexpasmantier/television/commit/f60b492383031fb23756b3b2f5d732d174e74033) _(cli)_ Add test that validates piping tv is working as expected by @alexpasmantier

- [42e2728](https://github.com/alexpasmantier/television/commit/42e272826915685fe71bd6d879e603e5fdeab86b) _(cli)_ New cli test suite by @lalvarezt

- [47b99c4](https://github.com/alexpasmantier/television/commit/47b99c43d067f9a51d8e12a14ff3c87ea7db1fae) _(e2e)_ Better pty tooling and more tests by @alexpasmantier

- [b780fa1](https://github.com/alexpasmantier/television/commit/b780fa1ba547ac2842bdcab60f963c0870b76626) _(e2e)_ Add proper e2e tests and pty-testing utils by @alexpasmantier

- [f6dcedc](https://github.com/alexpasmantier/television/commit/f6dcedc196f068f9077da28a93511a1e64749d6a) _(e2e)_ Fallback to a default value of 0 when failing to parse preview offset by @alexpasmantier

- [3b57710](https://github.com/alexpasmantier/television/commit/3b5771000622ee02bba414cadd4419d466fd8116) _(e2e)_ More end to end tests by @alexpasmantier

- [8d822cd](https://github.com/alexpasmantier/television/commit/8d822cd2fcfdc3a00c612e30674391426e988040) _(e2e)_ Add e2e tests for secondary cli commands (version, init, list-channels, ...) by @alexpasmantier

- [6662544](https://github.com/alexpasmantier/television/commit/666254498ee54b9ee09d01424b7382e0d30e7614) _(television)_ Add test to check channel keybindings precedence by @alexpasmantier

- [a59aab6](https://github.com/alexpasmantier/television/commit/a59aab67a9da94965432cdd800e207929ab3d28f) _(uncategorized)_ Add integration test for `--watch` by @alexpasmantier

- [dd832fc](https://github.com/alexpasmantier/television/commit/dd832fcfc9e5113f2a57924bc845b85ee6728aac) _(uncategorized)_ A cleaner integration tests directory structure by @alexpasmantier

- [fe8bdc6](https://github.com/alexpasmantier/television/commit/fe8bdc632b8e101fdf235ef24e68920ea52c4b0d) _(uncategorized)_ Add test to check cli overrides by @alexpasmantier

### ⚙️ Miscellaneous Tasks

- [f58e46c](https://github.com/alexpasmantier/television/commit/f58e46c40aca9a31003c2120bcd6772643d38bbb) _(changelog)_ Update changelog (auto) by @github-actions[bot] in [#591](https://github.com/alexpasmantier/television/pull/591)

- [d106ada](https://github.com/alexpasmantier/television/commit/d106adafc0a8f4d17bc4235e3bc439487db4a0b4) _(changelog)_ Update changelog (auto) by @github-actions[bot] in [#513](https://github.com/alexpasmantier/television/pull/513)

- [64c599e](https://github.com/alexpasmantier/television/commit/64c599ef103d18e852d1070c6b313800646f1940) _(changelog)_ Update changelog (auto) by @github-actions[bot] in [#491](https://github.com/alexpasmantier/television/pull/491)

- [a602dda](https://github.com/alexpasmantier/television/commit/a602dda34758f9f4a24f1c77b589216c12b9cfba) _(changelog)_ Update changelog (auto) by @github-actions[bot] in [#478](https://github.com/alexpasmantier/television/pull/478)

- [2e99fba](https://github.com/alexpasmantier/television/commit/2e99fba9c0dbe572727fca6f0a9593309f8cbe54) _(nix)_ Update sha of rust toolchain in flake.nix by @tukanoidd in [#530](https://github.com/alexpasmantier/television/pull/530)

- [738fe08](https://github.com/alexpasmantier/television/commit/738fe08fbb5fae9f1185b9980c7c344652b7b9d4) _(rust)_ Update rust edition to 2024 and version to 1.87 by @alexpasmantier in [#528](https://github.com/alexpasmantier/television/pull/528)

- [f6b2205](https://github.com/alexpasmantier/television/commit/f6b22051cdfbce8f6598c5d36c4b75887ff65998) _(tui-input)_ Add credit and license for `television/utils/input.rs` by @alexpasmantier in [#544](https://github.com/alexpasmantier/television/pull/544)

- [71582e5](https://github.com/alexpasmantier/television/commit/71582e559ddf84b1fb912aa1364fd91a9d5bf04e) _(uncategorized)_ Bump to 0.12.0 by @alexpasmantier

- [429bfae](https://github.com/alexpasmantier/television/commit/429bfaeb2f3dbbc7015213db1c2f12564615e5ca) _(uncategorized)_ Remove unused serde renames by @alexpasmantier

- [141d3e7](https://github.com/alexpasmantier/television/commit/141d3e7fb928c7020aa47240e97d9ff8dc0e753f) _(uncategorized)_ Update dependencies by @alexpasmantier

- [ab1efed](https://github.com/alexpasmantier/television/commit/ab1efed88de7c5d30c4f8ae2c1644ac207d31cc3) _(uncategorized)_ Remove stale FIXME comment by @alexpasmantier

- [11c2ef4](https://github.com/alexpasmantier/television/commit/11c2ef4eef77cbdf8c5676be3e3dcb6d9812bc03) _(uncategorized)_ Create github action workflow for pages by @alexpasmantier

- [0f6b29b](https://github.com/alexpasmantier/television/commit/0f6b29ba817f54da7c6cc694c21127c8588709a0) _(uncategorized)_ Add sponsorhips button to the repo by @alexpasmantier

### New Contributors

- @Ktoks made their first contribution in [#590](https://github.com/alexpasmantier/television/pull/590)
- @lalvarezt made their first contribution in [#588](https://github.com/alexpasmantier/television/pull/588)
- @kapobajza made their first contribution in [#568](https://github.com/alexpasmantier/television/pull/568)
- @domaschh made their first contribution
- @cr4ftx made their first contribution in [#496](https://github.com/alexpasmantier/television/pull/496)

**Full Changelog**: https://github.com/alexpasmantier/television/compare/0.11.9...0.12.0

<!-- generated by git-cliff -->
