# Binding System Architecture

## Overview

The Television binding system maps keys and events to actions using a dual-format parser (Television syntax + TOML) built on Pest. The system supports channel-specific bindings, action blocks, event bindings, and multi-source configuration loading.

```mermaid
graph TD
    A[Configuration Input] --> B{Parser Selection}

    B -->|Binding System| C1
    B -->|TOML| D1

    subgraph primary[" "]
        C1[parse_bindings]
        C2[Pest Grammar Processing]
        C3[Television AST Generation]
        C1 --> C2 --> C3
    end

    subgraph secondary[" "]
        D1[parse_toml_bindings]
        D2[TOML Grammar Processing]
        D3[TOML to AST Conversion]
        D1 --> D2 --> D3
    end

    C3 --> E[Unified AST]
    D3 --> E

    E --> F[Converter Layer]
    F --> G[Television Bindings]
    G --> H[Runtime Execution]

    B -->|Both Fail| I[Parse Error]
```

## Grammar Definition

### Core Grammar Structure

```pest
// Dual format support
file = { SOI ~ (bindings | toml_bindings)? ~ EOI }

// Television syntax
bindings = { "bindings" ~ "{" ~ binding* ~ "}" }
binding = {
    key_binding | event_binding | channel_binding |
    for_channels_binding | comment
}

// Binding types
key_binding = { key_sequence ~ "=>" ~ action_target ~ ";" }
event_binding = { "@" ~ event_name ~ "=>" ~ action_target ~ ";" }
channel_binding = { "channel" ~ string_literal ~ "{" ~ binding* ~ "}" }
for_channels_binding = { "for_channels" ~ "(" ~ string_literal ~ ")" ~ "{" ~ binding* ~ "}" }

// Action targets
action_target = { action_block | action_array | action_name }
action_block = { "{" ~ action_name ~ (";" ~ action_name)* ~ "}" }
action_array = { "[" ~ action_name ~ ("," ~ action_name)* ~ "]" }

// TOML format support
toml_bindings = { toml_section+ }
toml_section = { "[" ~ section_name ~ "]" ~ toml_entry* }
```

### Key Grammar Components

```pest
// Key specifications
key_sequence = { (modifier ~ "-")* ~ key_name }
modifier = { "ctrl" | "alt" | "shift" }
key_name = { named_key | function_key | character_key | mouse_event }

// Named keys
named_key = {
    "enter" | "esc" | "tab" | "space" | "backspace" | "delete" |
    "home" | "end" | "pageup" | "pagedown" | "insert" |
    "up" | "down" | "left" | "right" | "backtab"
}

// Function keys and mouse events
function_key = { "f" ~ number }
mouse_event = { "mouse-scroll-up" | "mouse-scroll-down" }

// Event specifications
event_name = {
    "start" | "load" | "result" | "one" | "zero" |
    "selection-change" | "resize"
}
```

## Hierarchical Configuration Merging

```mermaid
graph TB
    A["Embedded Defaults"] --> B["User Config"]
    B --> C["Channel Configs"]

    subgraph ABS["Multi-Source Loading"]
        D1["Explicit Files<br/>--bindings-file FILENAME"]
        D2["Custom Directory<br/>--bindings-dir DIRNAME"]
        D3["Default Directory<br/>$CONFIG/bindings.tvb<br/>$CONFIG/keybindings.toml"]
    end

    C --> D1
    C --> D2
    C --> D3

    E["Merge Bindings"] --> F["Final Configuration"]

    D1 --> E
    D2 --> E
    D3 --> E
```

## Action Block State Management

```mermaid
stateDiagram-v2
    [*] --> Received: ActionBlock arrives
    Received --> HashCheck: Calculate hash
    HashCheck --> Processed: Hash not found
    HashCheck --> Skipped: Hash exists
    Processed --> Queued: Mark hash, queue actions
    Queued --> [*]: Actions sent to channel
    Skipped --> [*]: Duplicate prevention
    note right of HashCheck
        Hash-based deduplication
    end note
    note left of Queued
        Individual actions sent
        via action_tx channel
    end note
    note left of Processed
        Action blocks processed
        sequentially to maintain
        execution order
    end note
```

## Grammar Extensions (not Implemented)

- 🔄 Parameterized action execution
- 🔄 Advanced pattern matching for channels
- 🔄 Conditional binding logic
- 🔄 Runtime binding modification
- 🔄 LSP support using AST span information
