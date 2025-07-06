def tv_smart_autocomplete [] {
    let current_prompt = (commandline)
    let cursor = (commandline get-cursor)
    let current_prompt = ($current_prompt | str substring 0..$cursor)
 
    let output = (tv --autocomplete-prompt $current_prompt --inline | str trim)

    if ($output | str length) > 0 {
        let needs_space = not ($current_prompt | str ends-with " ")
        let new_prompt = if $needs_space { $"($current_prompt) " + $output } else { $current_prompt + $output }
    }
    # Update the line editor with the new prompt

    if ($output | is-not-empty) {
        commandline edit --replace $output
        commandline set-cursor --end
    }
}

def tv_shell_history [] {
    let current_prompt = (commandline)
    let cursor = (commandline get-cursor)
    let current_prompt = ($current_prompt | str substring 0..$cursor)
    
    let output = (tv nu-history --input $current_prompt --inline | str trim)

    if ($output | is-not-empty) {
        commandline edit --replace $output
        commandline set-cursor --end
    }
}

# Bind custom keybindings

$env.config = (
  $env.config
  | upsert keybindings [
        {
            name: tv_completion,
            modifier: Control,
            keycode: char_t,
            mode: [vi_normal, vi_insert, emacs],
            event: {
                send: executehostcommand,
                cmd: "tv_smart_autocomplete"
            }
        }
        {
            name: tv_history,
            modifier: Control,
            keycode: char_r,
            mode: [vi_normal, vi_insert, emacs],
            event: {
                send: executehostcommand,
                cmd: "tv_shell_history"
            }
        }
    ]
)
