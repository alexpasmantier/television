_disable_bracketed_paste() {
    # Check if bracketed paste is defined, for compatibility with older versions
    if [[ -n $zle_bracketed_paste ]]; then
        print -nr ${zle_bracketed_paste[2]} >/dev/tty
    fi
}

_enable_bracketed_paste() {
    # Check if bracketed paste is defined, for compatibility with older versions
    if [[ -n $zle_bracketed_paste ]]; then
        print -nr ${zle_bracketed_paste[1]} >/dev/tty
    fi
}

_tv_smart_autocomplete() {
    emulate -L zsh
    zle -I

    _disable_bracketed_paste

    # prefix (lhs of cursor)
    local current_prompt
    current_prompt=$LBUFFER

    local output
    output=$(tv --autocomplete-prompt "$current_prompt" $* | tr '\n' ' ')

    if [[ -n $output ]]; then
        # suffix (rhs of cursor)
        local rhs=$RBUFFER
        # add a space if the prompt does not end with one
        [[ "${current_prompt}" != *" " ]] && current_prompt="${current_prompt} "

        LBUFFER=$current_prompt$output
        CURSOR=${#LBUFFER}
        RBUFFER=$rhs

        zle reset-prompt
        # uncomment this to automatically accept the line
        # (i.e. run the command without having to press enter twice)
        # zle accept-line
    fi

    _enable_bracketed_paste
}

_tv_shell_history() {
    emulate -L zsh
    zle -I

    _disable_bracketed_paste

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(history -n -1 0 | tv --input "$current_prompt" $*)

    if [[ -n $output ]]; then
        zle reset-prompt
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
