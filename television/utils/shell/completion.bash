function tv_smart_autocomplete() {
  local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"

  local output=$(tv --autocomplete-prompt "$current_prompt")

  if [[ -n $output ]]; then
    # add a space if the prompt does not end with one
    [[ "${current_prompt}" != *" " ]] && current_prompt="${current_prompt} "

    READLINE_LINE=$current_prompt$output
    READLINE_POINT=${#READLINE_LINE}
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

bind -x '"\C-t": tv_smart_autocomplete'
bind -x '"\C-r": tv_shell_history'
