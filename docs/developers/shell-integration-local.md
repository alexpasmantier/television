# Shell Integration: Developing locally

In order to develop locally on the shell integration scripts, here are a couple of steps to follow:

0. Clone the repo, make sure you're up to date and have the [just](https://github.com/casey/just) command runner installed.
1. Make your changes to any one of the shell scripts (`television/utils/shell/`)
2. Generate a dev version of the script by running:
   ```sh
   just generate-dev-shell-integration zsh
   ```
   (or `fish`, `bash`, etc. depending on the shell you are using)
3. Source the generated script in your shell:
   ```sh
   source dev_shell_integration.zsh
   ```
4. Test the changes by using the shell integration keybindings or commands in your terminal.
