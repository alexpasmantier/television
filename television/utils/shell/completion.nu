def tv_smart_autocomplete [] {
    let current_prompt = $env.PROMPT # Fetch the current prompt input

    let output = (tv --autocomplete-prompt $current_prompt | str trim)

    if ($output | str length) > 0 {
        let needs_space = not ($current_prompt | str ends-with " ")
        let new_prompt = if $needs_space { $"($current_prompt) " + $output } else { $current_prompt + $output }

        # Update the line editor with the new prompt
        $env.PROMPT = $new_prompt
    }
}

def tv_shell_history [] {
    let current_prompt = $env.PROMPT

    let output = (tv nu-history --input $current_prompt | str trim)

    if ($output | str length) > 0 {
        $env.PROMPT = $output
    }
}

# Bind custom keybindings
$env.config = ($env.config | upsert keybindings [
    { name: "tv_completion", modifier: none, keycode: "{tv_smart_autocomplete_keybinding}", mode: "vi_normal", action: { send: "tv_smart_autocomplete" } }
    { name: "tv_history", modifier: none, keycode: "{tv_shell_history_keybinding}", mode: "vi_normal", action: { send: "tv_shell_history" } }
])
