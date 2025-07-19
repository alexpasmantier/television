function __tv_parse_commandline --description 'Parse the current command line token and return split of existing filepath, and query'
    # credits to the junegunn/fzf project
    # https://github.com/junegunn/fzf/blob/9c1a47acf7453f9dad5905b7f23ad06e5195d51f/shell/key-bindings.fish#L53-L131

    set -l tv_query ''
    set -l prefix ''
    set -l dir '.'

    # Set variables containing the major and minor fish version numbers, using
    # a method compatible with all supported fish versions.
    set -l -- fish_major (string match -r -- '^\d+' $version)
    set -l -- fish_minor (string match -r -- '^\d+\.(\d+)' $version)[2]

    # fish v3.3.0 and newer: Don't use option prefix if " -- " is preceded.
    set -l -- match_regex '(?<tv_query>[\s\S]*?(?=\n?$)$)'
    set -l -- prefix_regex '^-[^\s=]+=|^-(?!-)\S'
    if test "$fish_major" -eq 3 -a "$fish_minor" -lt 3
        or string match -q -v -- '* -- *' (string sub -l (commandline -Cp) -- (commandline -p))
        set -- match_regex "(?<prefix>$prefix_regex)?$match_regex"
    end

    # Set $prefix and expanded $tv_query with preserved trailing newlines.
    if test "$fish_major" -ge 4
        # fish v4.0.0 and newer
        string match -q -r -- $match_regex (commandline --current-token --tokens-expanded | string collect -N)
    else if test "$fish_major" -eq 3 -a "$fish_minor" -ge 2
        # fish v3.2.0 - v3.7.1 (last v3)
        string match -q -r -- $match_regex (commandline --current-token --tokenize | string collect -N)
        eval set -- tv_query (string escape -n -- $tv_query | string replace -r -a '^\\\(?=~)|\\\(?=\$\w)' '')
    else
        # fish older than v3.2.0 (v3.1b1 - v3.1.2)
        set -l -- cl_token (commandline --current-token --tokenize | string collect -N)
        set -- prefix (string match -r -- $prefix_regex $cl_token)
        set -- tv_query (string replace -- "$prefix" '' $cl_token | string collect -N)
        eval set -- tv_query (string escape -n -- $tv_query | string replace -r -a '^\\\(?=~)|\\\(?=\$\w)|\\\n\\\n$' '')
    end

    if test -n "$tv_query"
        # Normalize path in $tv_query, set $dir to the longest existing directory.
        if test \( "$fish_major" -ge 4 \) -o \( "$fish_major" -eq 3 -a "$fish_minor" -ge 5 \)
            # fish v3.5.0 and newer
            set -- tv_query (path normalize -- $tv_query)
            set -- dir $tv_query
            while not path is -d $dir
                set -- dir (path dirname $dir)
            end
        else
            # fish older than v3.5.0 (v3.1b1 - v3.4.1)
            if test "$fish_major" -eq 3 -a "$fish_minor" -ge 2
                # fish v3.2.0 - v3.4.1
                string match -q -r -- '(?<tv_query>^[\s\S]*?(?=\n?$)$)' \
                    (string replace -r -a -- '(?<=/)/|(?<!^)/+(?!\n)$' '' $tv_query | string collect -N)
            else
                # fish v3.1b1 - v3.1.2
                set -- tv_query (string replace -r -a -- '(?<=/)/|(?<!^)/+(?!\n)$' '' $tv_query | string collect -N)
                eval set -- tv_query (string escape -n -- $tv_query | string replace -r '\\\n$' '')
            end
            set -- dir $tv_query
            while not test -d "$dir"
                set -- dir (dirname -z -- "$dir" | string split0)
            end
        end

        if not string match -q -- '.' $dir; or string match -q -r -- '^\./|^\.$' $tv_query
            # Strip $dir from $tv_query - preserve trailing newlines.
            if test "$fish_major" -ge 4
                # fish v4.0.0 and newer
                string match -q -r -- '^'(string escape --style=regex -- $dir)'/?(?<tv_query>[\s\S]*)' $tv_query
            else if test "$fish_major" -eq 3 -a "$fish_minor" -ge 2
                # fish v3.2.0 - v3.7.1 (last v3)
                string match -q -r -- '^/?(?<tv_query>[\s\S]*?(?=\n?$)$)' \
                    (string replace -- "$dir" '' $tv_query | string collect -N)
            else
                # fish older than v3.2.0 (v3.1b1 - v3.1.2)
                set -- tv_query (string replace -- "$dir" '' $tv_query | string collect -N)
                eval set -- tv_query (string escape -n -- $tv_query | string replace -r -a '^/?|\\\n$' '')
            end
        end
    end

    # Ensure $dir ends with a slash if it's a directory
    if test -d "$dir"; and not string match -q '*/$' -- $dir
        set dir "$dir/"
    end

    string escape -n -- "$dir" "$tv_query" "$prefix"

end

function tv_smart_autocomplete
    set -l commandline (__tv_parse_commandline)
    set -lx dir $commandline[1]
    set -l tv_query $commandline[2]

    # prefix (lhs of cursor)
    set -l current_prompt (commandline --current-process)

    # move to the next line so that the prompt is not overwritten
    printf "\n"

    if set -l result (tv $dir --autocomplete-prompt "$current_prompt" --input $tv_query --inline)
        # Remove last token from commandline.
        commandline -t ''

        # If dir is the current directory, i.e. './' , clear it.
        # If the pattern './foo' './bar' instead of 'foo' 'bar' is desired then comment out the check below
        if test "$dir" = "./"
            set dir ""
        end

        for i in $result
            commandline -t -- (string escape -- "$dir$i")' '
            # optional, if you want to replace '/home/foo/' with '~/', comment out above and uncomment below
            # commandline -t -- (string replace --all $HOME '~' -- (string escape -- "$dir$i"))' '
        end
    end

    # move the cursor back to the previous line
    printf "\033[A"

    commandline -f repaint
end

function tv_shell_history
    set -l current_prompt (commandline -cp)

    # move to the next line so that the prompt is not overwritten
    printf "\n"

    set -l output (tv fish-history --input "$current_prompt" --inline)

    if test -n "$output"
        commandline -r "$output"
    end
    # move the cursor back to the previous line
    printf "\033[A"
    commandline -f repaint
end

for mode in default insert
    bind --mode $mode {tv_smart_autocomplete_keybinding} tv_smart_autocomplete
    bind --mode $mode {tv_shell_history_keybinding} tv_shell_history
end
