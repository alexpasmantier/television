# Television PowerShell Integration
# Requires PSReadLine module (usually included by default in PowerShell 5.1+)

function Get-AvailableSpaceBelowPrompt {
    $windowHeight = [Console]::WindowHeight
    $windowTop = [Console]::WindowTop
    $cursorY = [Console]::CursorTop
    $spaceBelow = $windowHeight - ($cursorY - $windowTop)
    return $spaceBelow
}

function Invoke-TvSmartAutocomplete {
    <#
    .SYNOPSIS
        Smart autocomplete using television (tv) based on current command context
    #>
    [CmdletBinding()]
    param()

    # Get the current command line and cursor position
    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    # Get the part before cursor (left buffer)
    $lhs = $line.Substring(0, $cursor)
    $rhs = $line.Substring($cursor)

    # Separate lhs into words to get the last word
    $words = $lhs -split '\s+'
    # Handle trailing space - if last char is space, we're starting a new word
    if ($lhs.Length -gt 0 -and $lhs[-1] -match '\s') {
        $lastWord = ""
    } else {
        $lastWord = if ($words.Count -gt 0) { $words[-1] } else { "" }
    }

    # Call tv with autocomplete prompt
    try {
        # Debug file for cursor position tracking
        $debugFile = Join-Path $env:TEMP "tv-pwsh-debug.log"

        # Save the original prompt position
        $originalPromptY = [Console]::CursorTop
        $savedCursorX = [Console]::CursorLeft

        # skip a line to make room for TV's UI
        [Console]::WriteLine()

        # how much space left down
        $spaceBelow = Get-AvailableSpaceBelowPrompt
        $minTVHeight = 15
        $tvScrollLines = 0
        if ($spaceBelow -lt $minTVHeight) {
            # Not enough space below, scroll up
            $tvScrollLines = $minTVHeight - $spaceBelow
        }

        # Escape arguments properly for Windows command-line parsing
        # Backslashes before quotes need to be escaped to prevent them from escaping the quote
        $lhsEscaped = $lhs -replace '\\+$', '$0$0'  # Double trailing backslashes
        $lastWordEscaped = $lastWord -replace '\\+$', '$0$0'  # Double trailing backslashes

        # Use .NET Process class for explicit stream control
        # This ensures stdin comes from console, stdout is captured, and stderr (TUI) goes to console
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "tv"
        $psi.Arguments = "--no-status-bar --inline --autocomplete-prompt `"$lhsEscaped`" --input `"$lastWordEscaped`""
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $false  # Let TUI render to console
        $psi.RedirectStandardInput = $false  # Let process use console directly for input
        $psi.WorkingDirectory = $PWD.Path  # Use current PowerShell working directory

        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $psi
        $process.Start() | Out-Null

        # Read stdout
        $output = $process.StandardOutput.ReadToEnd().Trim()
        $process.WaitForExit()

        # Restore cursor to original position
        [Console]::CursorTop = $originalPromptY - $tvScrollLines
        [Console]::CursorLeft = $savedCursorX

        if ($output) {
            # TV returns the full completed path, so we need to remove the lastWord from lhs
            # to avoid duplication
            $lhsWithoutLastWord = if ($lastWord.Length -gt 0) {
                $lhs.Substring(0, $lhs.Length - $lastWord.Length)
            } else {
                $lhs
            }

            # Replace the buffer with the completion
            $newLine = $lhsWithoutLastWord + $output + $rhs
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $newLine)
        }
    }
    catch {
        # Print error if tv is not available or errors occur
        Write-Debug "TV autocomplete failed: $_"
    }
}

function Invoke-TvShellHistory {
    <#
    .SYNOPSIS
        Search shell history using television (tv)
    #>
    [CmdletBinding()]
    param()

    # Get the current command line and cursor position
    $line = $null
    $cursor = $null
    [Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

    # Get the part before cursor as search context
    $currentPrompt = $line.Substring(0, $cursor)

    # Call tv with history channel
    try {
        # Debug file for cursor position tracking
        $debugFile = Join-Path $env:TEMP "tv-pwsh-debug.log"

        # Save the original prompt position before any scrolling
        $originalPromptY = [Console]::CursorTop
        $savedCursorX = [Console]::CursorLeft

        # skip a line to make room for TV's UI
        [Console]::WriteLine()

        # how much space left down
        $spaceBelow = Get-AvailableSpaceBelowPrompt
        $minTVHeight = 15
        $tvScrollLines = 0
        if ($spaceBelow -lt $minTVHeight) {
            # Not enough space below, scroll up
            $tvScrollLines = $minTVHeight - $spaceBelow
        }

        # Escape arguments properly for Windows command-line parsing
        $currentPromptEscaped = $currentPrompt -replace '\\+$', '$0$0'  # Double trailing backslashes

        # Use .NET Process class for explicit stream control
        $psi = New-Object System.Diagnostics.ProcessStartInfo
        $psi.FileName = "tv"
        $psi.Arguments = "pwsh-history --inline --no-status-bar --input `"$currentPromptEscaped`""
        $psi.UseShellExecute = $false
        $psi.RedirectStandardOutput = $true
        $psi.RedirectStandardError = $false  # Let TUI render to console
        $psi.RedirectStandardInput = $false  # Let process use console directly for input
        $psi.WorkingDirectory = $PWD.Path  # Use current PowerShell working directory

        $process = New-Object System.Diagnostics.Process
        $process.StartInfo = $psi
        $process.Start() | Out-Null

        # Read stdout (the selected result)
        $output = $process.StandardOutput.ReadToEnd().Trim()
        $process.WaitForExit()

        # Restore cursor to original position
        [Console]::CursorTop = $originalPromptY - $tvScrollLines
        [Console]::CursorLeft = $savedCursorX

        if ($output) {
            # Replace entire line with selected history
            [Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $output)
        }
    }
    catch {
        # Print error if tv is not available or errors occur
        Write-Debug "TV shell history failed: $_"
    }
}

# Set up key bindings
Set-PSReadLineKeyHandler -Chord '{tv_smart_autocomplete_keybinding}' -ScriptBlock {
    Invoke-TvSmartAutocomplete
}

Set-PSReadLineKeyHandler -Chord '{tv_shell_history_keybinding}' -ScriptBlock {
    Invoke-TvShellHistory
}
