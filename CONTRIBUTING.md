# Contributing

First of all, thanks for considering contributing to this project. All contributions are welcome, whether they are bug reports, documentation improvements, feature requests, or pull requests.

If you're not sure where to start, take a look at the [Hot Topics](#hot-topics) section for some ideas on what you could work on.

# Contributing to television's documentation website

To contribute to the docs, please refer to [Contributing to docs](https://github.com/alexpasmantier/television/blob/main/website/CONTRIBUTING.md). This will guide you through the process of setting up the documentation environment and making changes.

## Getting started

### Prerequisites

These are pretty much the only things you need to have installed on your machine to get started with contributing to this project:

- the [Rust](https://www.rust-lang.org/tools/install) toolchain installed on your machine
- any working version of [Git](https://git-scm.com/downloads)
- the [just](https://github.com/casey/just) command runner

### Forking the repository and setting up the project

1. Click on the `Fork` button at the top right corner of the repository page to create a copy of the repository to your GitHub account.
2. Clone the forked repository to your local machine by running the following command in your terminal:
   ```shell
   git clone https://github.com/<your-username>/television.git
   ```
3. Navigate to the project directory and set up the upstream remote by running the following commands:
   ```shell
   cd television
   git remote add upstream https://github.com/alexpasmantier/television.git
   ```
4. Install the project dependencies by running the following command:
   ```shell
   just setup
   ```
5. Create a new branch for your feature or bug fix:
   ```shell
   git checkout -b <branch-name>
   ```
6. Make your changes and test them locally. Predefined commands are available to make your life simpler, using them spares some time and effort:
   ```shell
   just --list
   ```
7. Once you're all set, commit them to your branch:
   ```shell
   git add .
   git commit -m "Your commit message"
   ```
8. Push your changes to your forked repository:
   ```shell
   git push origin <branch-name>
   ```
9. If not done automatically, create a pull request by navigating to the original repository and clicking on the `New pull request` button.

### Developing locally

Before anything else (if not done already):

```shell
just setup
```

To run the application in debug mode while developing, with the ability to see logs and debug information:

```shell
just run
```

**Accessing the Logs:**

The logs are written to a file called `television.log` in a directory that depends on your operating system / configuration:

| Platform | Location                                                                                                     |
| -------- | ------------------------------------------------------------------------------------------------------------ |
| Linux    | `$XDG_DATA_HOME/television/television.log` or `$HOME/.local/share/television/television.log`                 |
| macOS    | `$XDG_DATA_HOME/television/television.log` or `$HOME/Library/Application\ Support/television/television.log` |
| Windows  | `%LocalAppData%\television\data\television.log`                                                              |

To check for linting and formatting issues (and fix them automatically), run:

```shell
just fix
```

To get a sense of the real thing and test how your patch would behave in production, run:

```shell
just b release

# or `just br`
# or `just build release`
```

Running the tests can be done with:

```shell
just test
```

### Contributing a new channel

Contributing a new channel is pretty straightforward.

1. create a new branch, add and commit your new channel's TOML file under `cable/unix` (or `cable/windows` depending on your usecase)
2. [optional] add a screenshot of the channel in `assets/channels/<os>/` (e.g. `assets/channels/unix/my_channel.png`).
3. push your commit and create a PR
4. the ci will automatically generate the documentation for your channel and pick up the screenshot if available.

## Hot Topics

- shell integration (autocomplete, keybindings)
- packaging for various linux package managers (dnf, ...)
- sorting options
- ansi parsing
- contributing new channels
- improving code documentation

## Troubleshooting

### `just test` fails with a Zig version error

If running the tests produces something like:

```
  src/build/Config.zig:69:17: error: root source file struct 'process' has no member named 'EnvMap'
  env: std.process.EnvMap,
       ~~~~~~~~~~~^~~~~~~
  /usr/lib/zig/std/process.zig:1:1: note: struct declared here
  const builtin = @import("builtin");
  ^~~~~
  referenced by:
      init: src/build/Config.zig:71:53
      build: build.zig:18:39
      7 reference(s) hidden; use '-freference-trace=9' to see all references
  src/build/zig.zig:13:9: error: Your Zig version v0.16.0 does not meet the required build version of v0.15.2
          @compileError(std.fmt.comptimePrint(
          ^~~~~~~~~~~~~
  build.zig:10:24: note: called at comptime here
      buildpkg.requireZig(minimumZigVersion);
      ~~~~~~~~~~~~~~~~~~~^~~~~~~~~~~~~~~~~~~

  thread 'main' panicked at ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libghostty-vt-sys-0.1.1/build.rs:132:5:
  zig build failed with status exit status: 2
  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

the test suite pulls in [`phantom-test`](https://crates.io/crates/phantom-test) for PTY-based TUI testing, which transitively builds [`libghostty-vt-sys`](https://crates.io/crates/libghostty-vt-sys) from source against a pinned [ghostty](https://github.com/ghostty-org/ghostty) commit. That commit currently requires Zig **0.15.2**, so a newer system Zig will fail the build.

The repo ships a `mise.toml` pinning Zig 0.15.2 for exactly this reason. If you already use [mise](https://mise.jdx.dev), run:

```shell
mise install
```

from the repo root and the right toolchain will be fetched and shimmed automatically — your system Zig stays untouched elsewhere.

If you don't have mise yet:

```shell
# Arch: `sudo pacman -S mise`, or:
curl https://mise.run | sh
echo 'eval "$(mise activate zsh)"' >> ~/.zshrc && exec zsh
mise install
```

Prefer a different tool? [zvm](https://github.com/tristanisham/zvm) and [zigup](https://github.com/marler8997/zigup) work just as well, or you can grab the [Zig 0.15.2 archive](https://ziglang.org/download/) and put it on `PATH` manually.
