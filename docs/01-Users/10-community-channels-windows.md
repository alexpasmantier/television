# 🍿 Community Channels (windows)

### _alias_

A channel to select from shell aliases

**Requirements:** _None_

**Code:** _alias.toml_

```toml
[metadata]
name = "alias"
description = "A channel to select from shell aliases"

[source]
command = "Get-Alias | %{ \"{0,-10} = {1,-10}\" -f $_.Name,$_.Definition }"
output = "{split:=:0|trim}"

```

---

### _dirs_

A channel to select from directories

**Requirements:** `fd`

**Code:** _dirs.toml_

```toml
[metadata]
name = "dirs"
description = "A channel to select from directories"
requirements = [ "fd",]

[source]
command = [ "fd -t d", "fd -t d --hidden",]

[preview]
command = "ls -l {}"

[keybindings]
shortcut = "f2"

```

---

### _docker-images_

A channel to select from Docker images

**Requirements:** `docker`, `jq`

**Code:** _docker-images.toml_

```toml
[metadata]
name = "docker-images"
description = "A channel to select from Docker images"
requirements = [ "docker", "jq",]

[source]
command = "docker images --format '{{.Repository}}:{{.Tag}} {{.ID}}'"
output = "{split: :-1}"

[preview]
command = "docker image inspect {split: :-1} | jq -C"

```

---

### _dotfiles_

A channel to select from your user's dotfiles

**Requirements:** `fd`, `bat`

**Code:** _dotfiles.toml_

```toml
[metadata]
name = "dotfiles"
description = "A channel to select from your user's dotfiles"
requirements = [ "fd", "bat",]

[source]
command = "fd -t f . \"$env:USERPROFILE\\AppData\\Roaming\\\""

[preview]
command = "bat -n --color=always {}"

```

---

### _env_

A channel to select from environment variables

**Requirements:** _None_

**Code:** _env.toml_

```toml
[metadata]
name = "env"
description = "A channel to select from environment variables"

[source]
command = "Get-ChildItem Env: | %{ \"{0,-30} = {1,-30}\" -f $_.Name,$_.Value }"
output = "{split:=:1..}"

[preview]
command = "echo '{split:=:1..}'"

[ui]
layout = "portrait"

[keybindings]
shortcut = "f3"

[ui.preview_panel]
size = 20
header = "{split:=:0|trim}"

```

---

### _files_

A channel to select files and directories

**Requirements:** `fd`, `bat`

**Code:** _files.toml_

```toml
[metadata]
name = "files"
description = "A channel to select files and directories"
requirements = [ "fd", "bat",]

[source]
command = [ "fd -t f", "fd -t f -H",]

[preview]
command = "bat -n --color=always {}"

[keybindings]
shortcut = "f1"

[preview.env]
BAT_THEME = "ansi"

```

---

### _git-branch_

A channel to select from git branches

**Requirements:** `git`

**Code:** _git-branch.toml_

```toml
[metadata]
name = "git-branch"
description = "A channel to select from git branches"
requirements = [ "git",]

[source]
command = "git --no-pager branch --all --format=\"%(refname:short)\""
output = "{split: :0}"

[preview]
command = "git show -p --stat --pretty=fuller --color=always {0}"

```

---

### _git-diff_

A channel to select files from git diff commands

**Requirements:** `git`

**Code:** _git-diff.toml_

```toml
[metadata]
name = "git-diff"
description = "A channel to select files from git diff commands"
requirements = [ "git",]

[source]
command = "git diff --name-only HEAD"

[preview]
command = "git diff HEAD --color=always -- {}"

```

---

### _git-log_

A channel to select from git log entries

**Requirements:** `git`

**Code:** _git-log.toml_

```toml
[metadata]
name = "git-log"
description = "A channel to select from git log entries"
requirements = [ "git",]

[source]
command = "git log --oneline --date=short --pretty=\"format:%h %s %an %cd\""
output = "{split: :0}"

[preview]
command = "git show -p --stat --pretty=fuller --color=always {0}"

```

---

### _git-reflog_

A channel to select from git reflog entries

**Requirements:** `git`

**Code:** _git-reflog.toml_

```toml
[metadata]
name = "git-reflog"
description = "A channel to select from git reflog entries"
requirements = [ "git",]

[source]
command = "git reflog"
output = "{split: :0}"

[preview]
command = "git show -p --stat --pretty=fuller --color=always {0}"

```

---

### _git-repos_

A channel to select from git repositories on your local machine.

This channel uses `fd` to find directories that contain a `.git` subdirectory, and then allows you to preview the git log of the selected repository.

**Requirements:** `fd`, `git`

**Code:** _git-repos.toml_

```toml
[metadata]
name = "git-repos"
requirements = [ "fd", "git",]
description = "A channel to select from git repositories on your local machine.\n\nThis channel uses `fd` to find directories that contain a `.git` subdirectory, and then allows you to preview the git log of the selected repository.\n"

[source]
command = "fd -g .git -HL -t d -d 10 --prune 'C:\\Users' --exec dirname '{}'"
display = "{split:\\\\:-1}"

[preview]
command = "cd '{}'; git log -n 200 --pretty=medium --all --graph --color"

```

---

### _pwsh-history_

A channel to select from your powershell history

**Requirements:** _None_

**Code:** _pwsh-history.toml_

```toml
[metadata]
name = "pwsh-history"
description = "A channel to select from your powershell history"

[source]
command = "Get-Content (Get-PSReadLineOption).HistorySavePath | Select-Object -Last 500"

```

---

### _text_

A channel to find and select text from files

**Requirements:** `rg`, `bat`

**Code:** _text.toml_

```toml
[metadata]
name = "text"
description = "A channel to find and select text from files"
requirements = [ "rg", "bat",]

[source]
command = "rg . --no-heading --line-number"
display = "[{split:\\::..2}]\t{split:\\::2}"
output = "{split:\\::..2}"

[preview]
command = "bat -n --color=always {split:\\::0}"
offset = "{split:\\::1}"

[preview.env]
BAT_THEME = "ansi"

[ui.preview_panel]
header = "{split:\\::..2}"

```

---
