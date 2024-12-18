_tv_search() {
    emulate -L zsh
    zle -I

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(tv --autocomplete-prompt "$current_prompt" $*)

    zle reset-prompt

    if [[ -n $output ]]; then
        RBUFFER=""
        LBUFFER=$current_prompt$output

        # uncomment this to automatically accept the line 
        # (i.e. run the command without having to press enter twice)
        # zle accept-line
    fi
}


zle -N tv-search _tv_search


bindkey '^T' tv-search

