# credits to the junegunn/fzf project
# https://github.com/junegunn/fzf/blob/d18c0bf6948b4707684fe77631aff26a17cbc4fa/shell/completion.zsh

_disable_bracketed_paste() {
    # Check if bracketed paste is defined, for compatibility with older versions
    if [[ -n $zle_bracketed_paste ]]; then
        print -nr ${zle_bracketed_paste[2]} >${TTY:-/dev/tty}
    fi
}

_enable_bracketed_paste() {
    # Check if bracketed paste is defined, for compatibility with older versions
    if [[ -n $zle_bracketed_paste ]]; then
        print -nr ${zle_bracketed_paste[1]} >${TTY:-/dev/tty}
    fi
}

__tv_path_completion() {
  local base lbuf suffix tail dir leftover matches
  base=$1
  lbuf=$2
  suffix=""
  tail=" "

  eval "base=$base" 2> /dev/null || return
  [[ $base = *"/"* ]] && dir="$base"
  while [ 1 ]; do
    if [[ -z "$dir" || -d ${dir} ]]; then
      leftover=${base/#"$dir"}
      leftover=${leftover/#\/}
      [ -z "$dir" ] && dir='.'
      [ "$dir" != "/" ] && dir="${dir/%\//}"
      zle -I
      matches=$(
        shift
        tv "$dir" --autocomplete-prompt "$lbuf" --inline --input "$leftover" < /dev/tty | while read -r item; do
          item="${item%$suffix}$suffix"
          dirP="$dir/"
          [[ $dirP = "./" ]] && dirP=""
          echo -n -E "$dirP${(q)item} "
        done
      )
      matches=${matches% }
      if [ -n "$matches" ]; then
        LBUFFER="$lbuf$matches$tail"
      fi
      zle reset-prompt
      break
    fi
    dir=$(dirname "$dir")
    dir=${dir%/}/
  done
}

_tv_smart_autocomplete() {
  _disable_bracketed_paste

  local tokens prefix trigger lbuf
  setopt localoptions noshwordsplit noksh_arrays noposixbuiltins

  # http://zsh.sourceforge.net/FAQ/zshfaq03.html
  # http://zsh.sourceforge.net/Doc/Release/Expansion.html#Parameter-Expansion-Flags
  tokens=(${(z)LBUFFER})
  if [ ${#tokens} -lt 1 ]; then
    zle ${fzf_default_completion:-expand-or-complete}
    return
  fi

  [[ ${LBUFFER[-1]} == ' ' ]] && tokens+=("")

  if [[ ${LBUFFER} = *"${tokens[-2]-}${tokens[-1]}" ]]; then
    tokens[-2]="${tokens[-2]-}${tokens[-1]}"
    tokens=(${tokens[0,-2]})
  fi

  lbuf=$LBUFFER
  prefix=${tokens[-1]}
  [ -n "${tokens[-1]}" ] && lbuf=${lbuf:0:-${#tokens[-1]}}

  __tv_path_completion "$prefix" "$lbuf"

  _enable_bracketed_paste
}

_tv_shell_history() {
    emulate -L zsh
    zle -I

    _disable_bracketed_paste

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(history -n -1 0 | tv --input "$current_prompt" --inline $*)

    zle reset-prompt
    if [[ -n $output ]]; then
        RBUFFER=""
        LBUFFER=$output

        # uncomment this to automatically accept the line
        # (i.e. run the command without having to press enter twice)
        # zle accept-line
    fi

    _enable_bracketed_paste
}


zle -N tv-smart-autocomplete _tv_smart_autocomplete
zle -N tv-shell-history _tv_shell_history


bindkey '{tv_smart_autocomplete_keybinding}' tv-smart-autocomplete
bindkey '{tv_shell_history_keybinding}' tv-shell-history
