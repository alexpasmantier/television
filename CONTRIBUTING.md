# Contributing

First of all, thanks for considering contributing to this project. All contributions are welcome, whether they are bug
reports, documentation improvements, feature requests, or pull requests.

If you're not sure where to start, take a look at the [hot topics](#hot-topics) section for some ideas on what you could work on.


## Getting started
### Prerequisites
These are pretty much the only things you need to have installed on your machine to get started with contributing to this project:
- the [Rust](https://www.rust-lang.org/tools/install) toolchain installed on your machine
- any working version of [Git](https://git-scm.com/downloads)

### Forking the repository and setting up the project
1. Click on the `Fork` button at the top right corner of the repository page to create a copy of the repository to your
   GitHub account.
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
    make setup
    ```
5. Create a new branch for your feature or bug fix:
   ```shell
   git checkout -b <branch-name>
   ```
6. Make your changes and commit them to your branch:
   ```shell
   git add .
   git commit -m "Your commit message"
   ```
7. Push your changes to your forked repository:
   ```shell
   git push origin <branch-name>
   ```
8. If not done automatically, create a pull request by navigating to the original repository and clicking on the `New pull request` button.


### Building the project
Before anything else:
```shell
make setup
```
To run the application in debug mode while developing, with the ability to see logs and debug information:
```shell
make run
```

**Accessing the Logs:**

The logs are written to a file called `television.log` in a directory that depends on your operating system /
configuration:
| Platform | Location |
|----------|----------|
| Linux | `$XDG_DATA_HOME/television/television.log` or `$HOME/.local/share/television/television.log` |
| macOS | `$XDG_DATA_HOME/television/television.log` or `$HOME/Library/Application\ Support/television/television.log` |
| Windows | `{FOLDERID_LocalAppData}\television\television.log` |

To build the project in debug mode, run the following command in the project directory:
```shell
make
```
or
```shell
make build
```

To build the project in release mode, run the following command in the project directory:
```shell
make release
```
Formatting the code
```shell
make format
```
Linting the code
```shell
make lint
```
Running the tests
```shell
make test
```

### Project structure
The project is laid out in several rust crates that are organized in the following way:
- `television`: the main binary crate that contains the CLI application
- `television_channels`: a library crate that contains the channel implementations
- `television_derive`: a library crate that contains the derive macros used in the project
- `television_fuzzy`: a library crate that contains the fuzzy matcher
- `television_previewers`: a library crate that contains the previewer implementations
- `television_utils`: a library crate that contains utility functions and types used in the project


### Contributing a new channel

`television` is built around the concept of _channels_.

From a technical standpoint, channels are structs that implement the `OnAir` trait defined in `crates/television-channels/src/channels.rs`.

They can be anything that can respond to a user query and return a result under the form of a list of entries. This means channels can be anything from conventional data sources you might want to search through (like files, git repositories, remote filesystems, environment variables etc.) to more exotic implementations that might include a REPL, a calculator, a web browser, search through your spotify library, your email, etc.

As mentioned in [Project structure](#project-structure) `television` uses [crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html) for its different subcomponents (_previewers_, _channels_, _utils_, etc).

When contributing a new channel, you should create a new module in the `television_channels` crate with a new struct for
your channel and ensure that it implements the `OnAir` trait defined in [crates/television-channels/src/channels.rs](crates/television-channels/src/channels.rs)

```rust
// crates/television-channels/src/channels/my_new_channel.rs

use television_channels::channels::OnAir;

pub struct MyNewChannel;

impl OnAir for MyNewChannel {
    // Implement the OnAir trait for your channel here
}
```

You should also add your channel to the `TelevisionChannel` enum in the `television_channels` crate.

```rust
// crates/television-channels/src/channels.rs



#[derive(ToUnitChannel, ToCliChannel, Broadcast)]
pub enum TelevisionChannel {
    // Other channels
    MyNewChannel,
}
```

☝️ There are built-in channels in `television` that you might want to draw inspiration from if need be, they're located at [crates/television-channels/src/channels](crates/television-channels/src/channels).

**TODO**: document transitions between channels and previewers


## Hot Topics
Here are some ideas for contributions that would be very welcome:

- `Customization`:
  - allow users to customize more of the UI (UI colors, layouts, etc.)
  - allow users to customize the behavior of the application (e.g. the default channel, fuzzy matching constants, channel heuristics, etc.)
- `Channels`:
  - new channel ideas:
    - shell history
    - git (commits, branches, status, diff, ...)
    - remote filesystems (s3, ...)
    - kubernetes resources (jobs, pods, deployments, services, ...)
    - recent directories
    - makefile commands
    - etc.
  - add more tests for existing channels
- `Previewers`:
  - new previewer ideas:
    - previewing text in documents (pdfs, archives, ...)
    - previewing images (actually already implemented but commented out)
    - remote files (s3, ...)
    - etc.
  - add more tests for existing previewers
- `Documentation`:
  - add more technical documentation to the project
    - general design of the TUI application
    - design of channels, previewers, transitions, etc.
    - how to contribute a new channel, previewer, etc.
  - more docstrings in the code
- `Performance`:
  - working on reducing coupling between the different crates in the project
  - working on reducing the number of allocations and copies in the code
- `Project`:
  - polish project configuration:
    - CI/CD
    - code formatting
    - linting



