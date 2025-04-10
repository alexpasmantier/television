function tv_smart_autocomplete() {
  local prompt_prefix="${READLINE_LINE:0:$READLINE_POINT}"
  local prompt_suffix="${READLINE_LINE:$READLINE_POINT}"

  local output=$(tv --autocomplete-prompt "$prompt_prefix" | tr '\n' ' ')

  if [[ -n $output ]]; then
    # add a space if the prompt does not end with one
    [[ "${prompt_prefix}" != *" " ]] && prompt_prefix="${prompt_prefix} "

    local lhs=$prompt_prefix$output
    READLINE_LINE=$lhs$prompt_suffix
    READLINE_POINT=${#lhs}
  fi
}

function tv_shell_history() {
  local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"

  local output=$(tv bash-history --input "$current_prompt")

  if [[ -n $output ]]; then
    READLINE_LINE=$output
    READLINE_POINT=${#READLINE_LINE}
  fi
}

bind -x '"{tv_smart_autocomplete_keybinding}": tv_smart_autocomplete'
bind -x '"{tv_shell_history_keybinding}": tv_shell_history'
