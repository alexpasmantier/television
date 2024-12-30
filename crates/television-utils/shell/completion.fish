function tv_smart_autocomplete
    set -l current_prompt (commandline -cp)

    set -l output (tv --autocomplete-prompt "$current_prompt")

    if test -n "$output"
        commandline -r "$current_prompt$output"
    end
end

function tv_shell_history
    set -l current_prompt (commandline -cp)

    set -l output (tv fish-history --input "$current_prompt")

    if test -n "$output"
        commandline -r "$output"
    end
end

bind \ct tv_smart_autocomplete
bind \cr tv_shell_history
