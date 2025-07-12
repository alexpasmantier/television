def tv_smart_autocomplete [] {
    let line = (commandline)
    let cursor = (commandline get-cursor)
    let lhs = ($line | str substring 0..$cursor)
    let rhs = ($line | str substring $cursor..)
    let output = (tv --inline --autocomplete-prompt $lhs | str trim)

    if ($output | str length) > 0 {
        let needs_space = not ($lhs | str ends-with " ")
        let lhs_with_space = if $needs_space { $"($lhs) " } else { $lhs }
        let new_line = $lhs_with_space + $output + $rhs
        let new_cursor = ($lhs_with_space + $output | str length)
        commandline edit --replace $new_line
        commandline set-cursor $new_cursor
    }
}

def tv_shell_history [] {
    let current_prompt = (commandline)
    let cursor = (commandline get-cursor)
    let current_prompt = ($current_prompt | str substring 0..$cursor)

    let output = (tv nu-history --inline --input $current_prompt | str trim)

    if ($output | is-not-empty) {
        commandline edit --replace $output
        commandline set-cursor --end
    }
}

# Bind custom keybindings
$env.config = (
  $env.config
  | upsert keybindings (
      $env.config.keybindings
      | append [
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
)
