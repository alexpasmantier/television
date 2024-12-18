_tv_search() {
    emulate -L zsh
    zle -I

    local current_prompt
    current_prompt=$LBUFFER

    local output

    output=$(tv dirs $*)

    zle reset-prompt

    if [[ -n $output ]]; then
        RBUFFER=""
        LBUFFER=$current_prompt$output
    fi
}


zle -N tv-search _tv_search


bindkey '^T' tv-search
