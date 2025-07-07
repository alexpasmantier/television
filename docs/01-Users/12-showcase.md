# Showcase

This section is meant as a community-driven list of user workflows and ways to use television in different contexts to inspire others.

### ⭐ VSCode extension

https://marketplace.visualstudio.com/items?itemName=alexpasmantier.television

### ⭐ Zed Editor File Finder

This is a drop-in file editor setup for the [zed editor](https://zed.dev/) leveraging television, a task, and keybinding.

![tv-zed-editor-compressed](https://github.com/user-attachments/assets/fc50f835-a282-438a-bbad-507320017765)

1. Install television
2. Add a task for a file finder (`zed: open tasks` command)

`tasks.json`

```jsonc
{
  "label": "File Finder",
  "command": "zed $(tv files)",
  "hide": "always",
  "allow_concurrent_runs": true,
  "use_new_terminal": true
},
```

3. Add keybinding to open file finder in center terminal (`zed: open keymap` command)

`keymap.json`

```jsonc
[
  {
    "bindings": {
      // replace file_finder::Toggle
      "cmd-p": [
        "task::Spawn",
        { "task_name": "File Finder", "reveal_target": "center" },
      ],
    },
  },
]
```

This should result in an interaction similar to the GIF above.

### ⭐ Vim/Neovim plugin

credit: https://github.com/prabirshrestha/tv.vim

![tv.vim](https://github.com/user-attachments/assets/c2a1dc2a-55d0-4b7b-9ac9-f04a5ac0cb33)

### ⭐ Zed Editor Live Grep

This is a drop-in project search setup for the [zed editor](https://zed.dev/) with similar setup to the [file finder](https://github.com/alexpasmantier/television/wiki/Showcase#--zed-editor-file-finder).

https://github.com/user-attachments/assets/5afb94af-f10a-4cac-9f51-0aa97b771bf0

1. Install television
2. Add a task for a file finder (zed: open tasks command)

`tasks.json`

```json
{
    "label": "Live Grep",
    "command": "zed $(tv text)",
    "hide": "always",
    "allow_concurrent_runs": true,
    "use_new_terminal": true,
},
```

3. Add keybinding to open file finder in center terminal (zed: open keymap command)

`keymap.json`

```json
{
  "bindings": {
    "space f w": [
      "task::Spawn",
      { "task_name": "Live Grep", "reveal_target": "center" }
    ]
  }
}
```

You can replace `space f w` with your keybinding

**Improve performance**

If there's a delay before the Live Grep window lauches you can optimize this by spawning it in a different shell, eg. `bash` or `sh`.|
In the example below zed will use sh to spawn the process:

`tasks.json`

```json
  {
    "label": "Live Grep",
    "command": "zed $(tv text)",
    "hide": "always",
    "allow_concurrent_runs": true,
    "use_new_terminal": true,
    "shell": {
      "program": "sh"
    }
  },
```

If you neeed further optimalization you can add commandline args like `--noediting`, `--norc`, `--noprofile` to the shell process:

`tasks.json`

```json
    "shell": {
      "with_arguments": {
        "program": "sh",
        "args": ["--noediting", "--norc", "--noprofile"]
      }
    }
```

or

`tasks.json`

```json
    "shell": {
      "with_arguments": {
        "program": "bash",
        "args": ["--norc", "--noprofile"]
      }
    }
```

> Note that you need to have `tv` in your `$PATH`

> This optimizations can also be applied to [file finder](https://github.com/alexpasmantier/television/wiki/Showcase#--zed-editor-file-finder).

Huge thanks to [eddiebergman](https://github.com/eddiebergman) and [thomdabrowski](https://github.com/thomdabrowski) for these optimizations, [original thread](https://github.com/zed-industries/zed/discussions/22581#discussioncomment-11860946)
