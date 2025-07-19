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
| Windows  | `{FOLDERID_LocalAppData}\television\television.log`                                                          |

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


## Hot Topics

- contributing new channels
- improving code documentation
- shell integration (autocomplete, keybindings)
- packaging for various linux package managers (dnf, ...)
- sorting options
