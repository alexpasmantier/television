# tv-completion.zsh - A fuzzy finder script using `tv`
# This script provides fuzzy autocompletion for commands, paths, hosts, processes, and more.

# Set the completion trigger (default: '**')
: "${TV_COMPLETION_TRIGGER=**}"

# Options for `tv` during completion
: "${TV_COMPLETION_OPTS=--interactive}"
: "${TV_COMPLETION_PATH_OPTS=}"     # Customize for file path search
: "${TV_COMPLETION_DIR_OPTS=}"      # Customize for directory search

# Function to restore Zsh options
_tv_restore_options() {
  setopt "$@" 2>/dev/null
}

# Custom file path generator
_tv_compgen_path() {
  find . -type f 2>/dev/null
}

# Custom directory generator
_tv_compgen_dir() {
  find . -type d 2>/dev/null
}

# General file and directory completion
_tv_path_completion() {
  local result
  result=$(tv $TV_COMPLETION_PATH_OPTS)
  [[ -n $result ]] && zle -I && LBUFFER+="$result"
}

_tv_dir_completion() {
  local result
  result=$(tv 'dirs' $TV_COMPLETION_DIR_OPTS)
  echo $result
  [[ -n $result ]] && zle -I && LBUFFER+="$result"
}

# Generic fuzzy completion function
_tv_completion() {
  local cmd_word="$1"
  local result

  case "$cmd_word" in
    export|unset)
      result=$(printenv | cut -d= -f1 | tv $TV_COMPLETION_OPTS)
      ;;
    *)
      if [[ -d ${LBUFFER##* } ]]; then
        _tv_dir_completion
      else
        _tv_path_completion
      fi
      ;;
  esac

  [[ -n $result ]] && zle -I && LBUFFER+="$result"
}

# Hook into Zsh completion system
zle -C tv-completion complete-word _tv_completion
zle -N _tv_path_completion
zle -N _tv_dir_completion
bindkey '^I' tv-completion

# Initialize tv completion trigger
zstyle ':completion:*' completer _tv_completion

# Cleanup options when done
{ setopt localoptions; local _tv_restore_options } always {
  _tv_restore_options "$@"
}

