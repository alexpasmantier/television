function tv_smart_autocomplete() {
  # prefix (lhs of cursor)
  local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"

  local output=$(tv --autocomplete-prompt "$current_prompt" --inline | tr '\n' ' ')

  if [[ -n $output ]]; then
    # suffix (rhs of cursor)
    local rhs="${READLINE_LINE:$READLINE_POINT}"
    # add a space if the prompt does not end with one
    [[ "${current_prompt}" != *" " ]] && current_prompt="${current_prompt} "

    local lhs=$current_prompt$output
    READLINE_LINE=$lhs$rhs
    READLINE_POINT=${#lhs}
  fi
}

function tv_shell_history() {
  local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"

  local output=$(tv bash-history --input "$current_prompt" --inline)

  if [[ -n $output ]]; then
    READLINE_LINE=$output
    READLINE_POINT=${#READLINE_LINE}
  fi
}

bind -x '"{tv_smart_autocomplete_keybinding}": tv_smart_autocomplete'
bind -x '"{tv_shell_history_keybinding}": tv_shell_history'
