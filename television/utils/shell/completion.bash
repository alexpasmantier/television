_disable_bracketed_paste() {
    # Disable bracketed paste mode to prevent unwanted escape sequences
    printf '\e[?2004l' > /dev/tty
}

_enable_bracketed_paste() {
    # Re-enable bracketed paste mode
    printf '\e[?2004h' > /dev/tty
}

__tv_path_completion() {
    local base="$1"
    local lbuf="$2"
    local suffix=""
    local tail=" "
    local dir leftover matches

    # Evaluate the base path (handle ~, variables, etc.)
    eval "base=\"$base\"" 2>/dev/null || return

    # Extract directory part if base contains a slash
    [[ "$base" == *"/"* ]] && dir="$base"
    
    while true; do
        if [[ -z "$dir" || -d "$dir" ]]; then
            # Calculate leftover part (what comes after the directory)
            leftover="${base#"$dir"}"
            leftover="${leftover#/}"
            
            # Set default directory if empty
            [[ -z "$dir" ]] && dir='.'
            
            # Remove trailing slash unless it's root
            [[ "$dir" != "/" ]] && dir="${dir%/}"

            # move to the next line so that the prompt is not overwritten
            printf "\n"
            
            # Call tv with proper arguments and process output
            matches=$(
                tv "$dir" --autocomplete-prompt "$lbuf" --inline --input "$leftover" < /dev/tty | while IFS= read -r item; do
                    item="${item%$suffix}$suffix"
                    dirP="$dir/"
                    [[ "$dirP" == "./" ]] && dirP=""
                    # Quote the item to handle special characters
                    printf '%s ' "$dirP$(printf '%q' "$item")"
                done
            )
            
            # Remove trailing space
            matches="${matches% }"
            
            if [[ -n "$matches" ]]; then
                # Update readline buffer
                local new_line="$lbuf$matches$tail"
                local rhs="${READLINE_LINE:$READLINE_POINT}"
                READLINE_LINE="$new_line$rhs"
                READLINE_POINT=${#new_line}
            fi
            # move the cursor back to the previous line
            printf "\033[A"

            break
        fi
        
        # Move up one directory level
        dir=$(dirname "$dir")
        dir="${dir%/}/"
    done
}

tv_smart_autocomplete() {
    _disable_bracketed_paste

    local tokens prefix lbuf
    local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"
    
    # Split the current prompt into tokens
    # This is a simplified version of zsh's word splitting
    read -ra tokens <<< "$current_prompt"
    
    if [[ ${#tokens[@]} -lt 1 ]]; then
        # Fall back to default completion if no tokens
        _enable_bracketed_paste
        return
    fi

    # Handle trailing space
    [[ "${READLINE_LINE:$((READLINE_POINT-1)):1}" == " " ]] && tokens+=("")

    # Get the last token as prefix
    prefix="${tokens[-1]}"
    
    # Calculate lbuf (everything except the last token)
    if [[ -n "$prefix" ]]; then
        lbuf="${current_prompt:0:$((${#current_prompt} - ${#prefix}))}"
    else
        lbuf="$current_prompt"
    fi

    __tv_path_completion "$prefix" "$lbuf"

    _enable_bracketed_paste
}

tv_shell_history() {
    _disable_bracketed_paste

    local current_prompt="${READLINE_LINE:0:$READLINE_POINT}"
    local output

    # move to the next line so that the prompt is not overwritten
    printf "\n"

    # Get history using tv with the same arguments as zsh version
    output=$(tv bash-history --input "$current_prompt" --inline)

    if [[ -n "$output" ]]; then
        # Clear the right side of cursor and set new line
        READLINE_LINE="$output"
        READLINE_POINT=${#READLINE_LINE}

        # Uncomment this to automatically accept the line
        # (i.e. run the command without having to press enter twice)
        # accept-line() { echo; }; accept-line
    fi

    # move the cursor back to the previous line
    printf "\033[A"

    _enable_bracketed_paste
}

# Bind the functions to key combinations
bind -x '"{tv_smart_autocomplete_keybinding}": tv_smart_autocomplete'
bind -x '"{tv_shell_history_keybinding}": tv_shell_history'
