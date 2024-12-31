_tv_smart_autocomplete() {
    emulate -L zsh
    zle -I

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(tv --autocomplete-prompt "$current_prompt" $*)


    if [[ -n $output ]]; then
        zle reset-prompt
        RBUFFER=""
        # add a space if the prompt does not end with one
        [[ "${current_prompt}" != *" " ]] && current_prompt="${current_prompt} "
        LBUFFER=$current_prompt$output

        # uncomment this to automatically accept the line 
        # (i.e. run the command without having to press enter twice)
        # zle accept-line
    fi
}

_tv_shell_history() {
    emulate -L zsh
    zle -I

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(tv zsh-history --input "$current_prompt" $*)


    if [[ -n $output ]]; then
        zle reset-prompt
        RBUFFER=""
        LBUFFER=$output

        # uncomment this to automatically accept the line 
        # (i.e. run the command without having to press enter twice)
        # zle accept-line
    fi
}


zle -N tv-smart-autocomplete _tv_smart_autocomplete
zle -N tv-shell-history _tv_shell_history


bindkey '^T' tv-smart-autocomplete
bindkey '^R' tv-shell-history

