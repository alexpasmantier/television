if $SHELL_TYPE in ("best", "prompt_toolkit", "prompt-toolkit", "ptk"):
    import os
    import shlex
    import subprocess

    from prompt_toolkit.application import run_in_terminal
    from prompt_toolkit.document import Document
    from xonsh.events import events

    def _tv_replace_buffer(buffer, text, cursor_position=None):
        if cursor_position is None:
            cursor_position = len(text)
        buffer.document = Document(text, cursor_position=cursor_position)

    def _tv_current_token(text):
        token_start = 0
        quote = None
        escaped = False

        for index, char in enumerate(text):
            if escaped:
                escaped = False
                continue
            if char == "\\":
                escaped = True
                continue
            if quote is not None:
                if char == quote:
                    quote = None
                continue
            if char in ("'", '"'):
                quote = char
                continue
            if char.isspace():
                token_start = index + 1

        return token_start, text[token_start:]

    def _tv_unquote_token(token):
        chars = []
        quote = None
        escaped = False

        for char in token:
            if escaped:
                chars.append(char)
                escaped = False
                continue
            if char == "\\":
                escaped = True
                continue
            if quote is not None:
                if char == quote:
                    quote = None
                else:
                    chars.append(char)
                continue
            if char in ("'", '"'):
                quote = char
                continue
            chars.append(char)

        if escaped:
            chars.append("\\")

        return "".join(chars)

    def _tv_path_parts(token):
        unquoted_token = _tv_unquote_token(token)
        if "/" not in unquoted_token:
            return ".", "", unquoted_token

        display_dir = (
            unquoted_token
            if unquoted_token.endswith("/")
            else unquoted_token.rsplit("/", 1)[0] + "/"
        )
        command_dir = os.path.expandvars(os.path.expanduser(display_dir))

        while command_dir and not os.path.isdir(command_dir):
            parent = os.path.dirname(command_dir.rstrip(os.sep))
            if parent == command_dir:
                command_dir = "."
                display_dir = ""
                break
            command_dir = parent
            stripped = display_dir.rstrip("/")
            display_dir = stripped.rsplit("/", 1)[0] + "/" if "/" in stripped else ""

        query = unquoted_token[len(display_dir):]
        return command_dir or ".", display_dir, query

    def _tv_run(args, input_text=None):
        try:
            result = subprocess.run(
                args,
                input=input_text,
                stdout=subprocess.PIPE,
                text=True,
                check=False,
            )
        except FileNotFoundError:
            return ""

        if result.returncode != 0:
            return ""
        return result.stdout.strip()

    def _tv_smart_autocomplete_for_buffer(buffer):
        text = buffer.text
        cursor = buffer.cursor_position
        left = text[:cursor]
        right = text[cursor:]
        token_start, token = _tv_current_token(left)
        prompt = left[:token_start]
        directory, display_dir, query = _tv_path_parts(token)

        output = _tv_run([
            "tv",
            directory,
            "--autocomplete-prompt",
            prompt,
            "--input",
            query,
            "--no-status-bar",
        ])

        if not output:
            return

        matches = []
        for line in output.splitlines():
            if line:
                matches.append(shlex.quote(display_dir + line))

        if not matches:
            return

        new_left = prompt + " ".join(matches) + " "
        _tv_replace_buffer(buffer, new_left + right, len(new_left))

    def _tv_history_input():
        history = __xonsh__.history
        if history is None:
            return ""

        try:
            items = history.all_items(newest_first=True)
        except Exception:
            items = history.items(newest_first=True)

        seen = set()
        commands = []
        for item in items:
            command = item.get("inp") or item.get("cmd") or ""
            command = command.strip()
            if command and command not in seen:
                seen.add(command)
                commands.append(command)

        return "\n".join(commands)

    def _tv_shell_history_for_buffer(buffer):
        current_prompt = buffer.text[:buffer.cursor_position]
        output = _tv_run(
            ["tv", "--input", current_prompt, "--no-status-bar"],
            _tv_history_input(),
        )

        if output:
            _tv_replace_buffer(buffer, output)

    def _tv_run_in_terminal(event, callback):
        try:
            event.app.output.responds_to_cpr = False
        except Exception:
            pass
        run_in_terminal(callback)

    def tv_smart_autocomplete(event):
        _tv_run_in_terminal(
            event,
            lambda: _tv_smart_autocomplete_for_buffer(event.current_buffer),
        )

    def tv_shell_history(event):
        _tv_run_in_terminal(
            event,
            lambda: _tv_shell_history_for_buffer(event.current_buffer),
        )

    @events.on_ptk_create
    def tv_keybindings(bindings, **kw):
        @bindings.add("{tv_smart_autocomplete_keybinding}", eager=True)
        def _tv_smart_autocomplete(event):
            tv_smart_autocomplete(event)

        @bindings.add("{tv_shell_history_keybinding}", eager=True)
        def _tv_shell_history(event):
            tv_shell_history(event)
