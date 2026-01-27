# Template System

Television uses a powerful template system based on [string-pipeline](https://docs.rs/string_pipeline) for dynamic formatting of entries, previews, and outputs. This guide covers everything you need to know to master templating.

## Where Templates Are Used

Templates appear in several channel fields:
- `source.display`: Format how entries appear in the results list
- `source.output`: Format the final output when an entry is selected
- `preview.command`: Build the preview command
- `preview.header/footer`: Dynamic preview panel headers/footers
- `actions.*.command`: Format action commands

## Basic Placeholders

### The `{}` Placeholder

The simplest template - represents the entire entry:

```toml
[preview]
command = "cat '{}'"  # Preview the selected file
```

### Positional Placeholders

Access parts of a delimited entry by position:

```toml
# Entry: "file.txt:42:error message"
[preview]
command = "bat -H {1} '{0}'"  # Open file.txt, highlight line 42
```

- `{0}`: First field (`file.txt`)
- `{1}`: Second field (`42`)
- `{2}`: Third field (`error message`)

The default delimiter is `:`, but you can use any delimiter with `split`.

## The Split Operation

Split entries on custom delimiters:

```toml
# Entry: "path/file.txt|42|info"
[source]
output = "{split:|:0}"  # Outputs: path/file.txt
```

### Split Syntax

```
{split:DELIMITER:INDEX_OR_RANGE}
```

**Single index:**
```
{split:,:0}   # First element
{split:,:1}   # Second element
{split:,:-1}  # Last element
{split:,:-2}  # Second to last
```

**Ranges:**
```
{split:,:..}    # All elements (joined by delimiter)
{split:,:1..}   # From index 1 to end
{split:,:..2}   # From start to index 2 (exclusive)
{split:,:1..3}  # From index 1 to 3 (exclusive)
{split:,:1..-1} # From index 1 to second-to-last
```

### Split Examples

Given entry: `"a,b,c,d,e"`

| Template | Result |
|----------|--------|
| `{split:,:0}` | `a` |
| `{split:,:2}` | `c` |
| `{split:,:-1}` | `e` |
| `{split:,:1..3}` | `b,c` |
| `{split:,:2..}` | `c,d,e` |
| `{split:,:..2}` | `a,b` |

## Stripping ANSI Codes

Remove ANSI escape sequences (colors) from entries:

```toml
[source]
output = "{strip_ansi}"  # Clean output without color codes
```

Useful when source commands output colored text but you need plain text for further processing.

## Pipelines

Chain multiple operations with `|`:

```toml
# Entry: "\x1b[31mpath/file.txt:42:error\x1b[0m"
[source]
output = "{strip_ansi|split:\\::0}"  # Outputs: path/file.txt
```

Operations execute left to right.

## String Transformations

### Case Transformations

```
{upper}      # UPPERCASE
{lower}      # lowercase
{capitalize} # Capitalize first letter
```

### Trimming

```
{trim}       # Remove leading/trailing whitespace
{trim_start} # Remove leading whitespace
{trim_end}   # Remove trailing whitespace
```

### Prefix and Suffix

```
{prepend:PREFIX}  # Add prefix
{append:SUFFIX}   # Add suffix
```

**Example:**
```toml
[source]
display = "{prepend:> |append: <}"  # Entry "foo" becomes "> foo <"
```

### Padding

```
{pad:WIDTH:CHAR:DIRECTION}
```

- `WIDTH`: Target width
- `CHAR`: Padding character
- `DIRECTION`: `left`, `right`, or `center`

**Example:**
```toml
{pad:10:0:left}   # "42" becomes "0000000042"
{pad:10: :center} # "foo" becomes "   foo    "
```

## Regular Expressions

### Extract with Regex

```
{regex_extract:PATTERN}
{regex_extract:PATTERN:GROUP}
```

**Examples:**
```toml
# Extract numbers
{regex_extract:\d+}         # "file123.txt" -> "123"

# Extract capture group
{regex_extract:v(\d+):1}    # "v2.3.4" -> "2"
```

### Replace with Regex

```
{regex_replace:PATTERN:REPLACEMENT}
```

**Example:**
```toml
{regex_replace:\s+:_}  # "hello world" -> "hello_world"
```

## Working with Collections

When processing multiple entries (e.g., multi-select), use collection operations:

### Map

Apply a transformation to each element:

```toml
{split:,:..|map:{trim|upper}}
# "a, b, c" -> "A,B,C"
```

### Filter

Keep elements matching a pattern:

```toml
{split:,:..|filter:\.py$}
# "a.py,b.txt,c.py" -> "a.py,c.py"
```

### Sort

Sort elements:

```toml
{split:,:..|sort}       # Ascending
{split:,:..|sort:desc}  # Descending
```

### Join

Join elements with a custom delimiter:

```toml
{split:,:..|join:\n}  # Join with newlines
```

## Real-World Examples

### Git Log Channel

```toml
[source]
command = "git log --oneline --color=always"
output = "{strip_ansi|split: :0}"  # Extract commit hash
ansi = true

[preview]
command = "git show --color=always '{strip_ansi|split: :0}'"
```

### Text Search (ripgrep)

```toml
[source]
command = "rg --line-number --no-heading --color=always ."
output = "{strip_ansi|split:\\::..2}"  # file:line
ansi = true

[preview]
command = "bat -H '{split:\\::1}' --color=always '{split:\\::0}'"
offset = "{split:\\::1}"  # Scroll to matching line
```

### Process Management

```toml
# Entry: "12345  user  /usr/bin/process"
[source]
command = "ps aux"
display = "{split: :..3}"  # Show PID, user, command

[actions.kill]
command = "kill -9 '{split: :0}'"  # Kill by PID
```

### Docker with Tabs

```toml
[source]
command = "docker ps --format '{{.ID}}\\t{{.Names}}\\t{{.Status}}'"

[preview]
command = "docker logs '{split:\\t:0}'"  # Use container ID

[source]
display = "{split:\\t:1} ({split:\\t:2})"  # Show: name (status)
```

## Complex Pipeline Example

Transform a complex entry step by step:

```toml
# Input: "  \x1b[31mapp.py,readme.md,test.py\x1b[0m  "
# Goal: Filter .py files, sort, format as bullet list

{trim|strip_ansi|split:,:..|filter:\.py$|sort|map:{prepend:• }|join:\n}

# Step by step:
# 1. trim         -> "\x1b[31mapp.py,readme.md,test.py\x1b[0m"
# 2. strip_ansi   -> "app.py,readme.md,test.py"
# 3. split:,:..   -> ["app.py", "readme.md", "test.py"]
# 4. filter:\.py$ -> ["app.py", "test.py"]
# 5. sort         -> ["app.py", "test.py"]
# 6. map:prepend  -> ["• app.py", "• test.py"]
# 7. join:\n      -> "• app.py\n• test.py"
```

## Escaping Special Characters

- Backslash in delimiters: `\\:` for literal `:`
- Tab character: `\\t`
- Newline: `\\n`

## Debugging Tips

1. **Start simple**: Begin with `{}` and add operations one at a time
2. **Test incrementally**: Use `--source-command` to test templates interactively
3. **Check ANSI**: If colors aren't appearing correctly, try `strip_ansi`
4. **Verify delimiters**: Print raw output to see actual separators

```sh
# Debug a template
tv --source-command "echo 'a:b:c'" --preview-command "echo 'Field 0: {split:\\::0}, Field 1: {split:\\::1}'"
```

## Reference

For the complete template syntax specification, see the [string-pipeline documentation](https://docs.rs/string_pipeline).
