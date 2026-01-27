# Architecture Documentation

_NOTE: what follows has mostly been assembled using AI as an experiment and as a basis for further improvements. @lalvarezt and I (@alexpasmantier) have been proofreading it to make sure all the information is technically correct and really reflects the code's architecture, so that **other developers may rely on it as a technical source of truth** when getting started with the repo._

## Overview

Television is a terminal fuzzy finder built with Rust. It uses async/await and separate loops for event handling, rendering, and background tasks to stay responsive.

## High-Level Architecture

```
  ┌──────────────┐    ┌──────────────┐    ┌─────────────┐
  │ CLI & Config  │───►│ Application  │───►│   Output    │
  │              │    │ Orchestrator │    │             │
  └──────────────┘    └──────────────┘    └─────────────┘
                             │
                             ▼
    ┌─────────────────────────────────────────────────┐
    │                   Event Loops                   │
    │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
    │ │ Event Loop  │ │ Render Loop │ │ Watch Timer │ │
    │ └─────────────┘ └─────────────┘ └─────────────┘ │
    └─────────────────────────────────────────────────┘
                            │
                            ▼
    ┌─────────────────────────────────────────────────┐
    │                 Core Components                 │
    │ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
    │ │  Television │ │  Channels   │ │  Previewer  │ │
    │ │   (State)   │ │  (Sources)  │ │             │ │
    │ └─────────────┘ └─────────────┘ └─────────────┘ │
    └─────────────────────────────────────────────────┘
```

## How It Works

### 1. Startup

```mermaid
sequenceDiagram
    participant CLI
    participant Config
    participant Cable
    participant App
    participant Loops

    CLI->>Config: Parse args & load config files
    Config->>Cable: Load channel definitions
    Cable->>App: Initialize with merged config
    App->>Loops: Start event, render, and watch loops
    Loops->>App: Begin main event loop
```

### 2. Runtime Event Flow

```mermaid
flowchart TD
    A[User Input] --> B[Event Loop]
    B --> C[Convert to Action]
    C --> D[Action Channel]
    D --> E[App Handler]
    E --> F{Action Type}

    F -->|Input| G[Update Television State]
    F -->|Navigation| H[Update Picker]
    F -->|Render| I[Send Render Task]
    F -->|Channel Switch| J[Reload Channel]

    G --> K[Trigger Render]
    H --> K
    I --> L[Render Loop]
    J --> M[Channel Reload]

    L --> N[Update Terminal]
    M --> O[Update Results]
    O --> K
    K --> L
```

## Core Components

### Application Orchestrator (`app.rs`)

```mermaid
graph TB
    A[main.rs] --> B[App]

    B --> C[Event Loop]
    B --> D[Render Loop]
    B --> E[Watch Timer]

    B --> F[Television Core]
    F --> G[Channels]
    F --> H[Previewer]

    C -->|Events| B
    D -->|UI State| B
    E -->|Timer Actions| B

    B -->|Actions| I[Action Handler]
    I -->|Render Tasks| D
    I -->|State Updates| F
```

The main app that coordinates everything:

- **What it does:**
  - Manages app state and lifecycle
  - Routes messages between loops using async channels
  - Handles actions and state changes
  - Starts and stops components

- **Key channels:**
  - `action_tx/rx`: Actions from events to main loop
  - `render_tx/rx`: Rendering tasks to render loop
  - `event_rx`: Events from event loop
  - `ui_state_tx/rx`: UI state feedback from render loop

### Event System

#### Event Loop (`loops/event_loop.rs`)

```mermaid
flowchart LR
A[Raw Event] --> B[Event Loop]
B --> C{Event Type}
C -->|Keyboard| D[Key Mapping]
C -->|Mouse| E[Mouse Handler]
C -->|System| F[System Handler]
D --> G[Action]
E --> G
F --> G
G --> H[Action Channel]
H --> I[App Handler]

```

- **Purpose:** Handles keyboard input, mouse events, and system signals
- **Input:** Key presses, mouse clicks, terminal resize, Ctrl+C
- **Output:** Events sent to main loop
- **Features:**
  - Non-blocking event reading
  - Clean shutdown handling
  - Regular ticks for animations

#### Actions (`action.rs`)

All user interactions become actions:

```rust
pub enum Action {
    // Input actions
    AddInputChar(char),
    DeletePrevChar,

    // Navigation actions
    SelectNextEntry,
    SelectPrevEntry,

    // Application actions
    ConfirmSelection,
    ToggleRemoteControl,
    Render,

    // System actions
    Resize(u16, u16),
    Quit,
}
```

### Television Core (`television.rs`)

The main state manager:

- **What it tracks:**
  - Current mode (Channel vs RemoteControl)
  - Search pattern and matching settings
  - Selected entries and picker state
  - Preview state and handles

- **What it does:**
  - Pattern matching and filtering
  - Entry selection and multi-selection
  - Channel switching and mode changes
  - Preview coordination

### Channel System

```mermaid
stateDiagram-v2
    [*] --> Loading: Channel Selected
    Loading --> Ready: Source Command Complete
    Ready --> Filtering: User Input
    Ready --> [*]: Channel Switch
    Filtering --> Ready: Results Updated
    Ready --> Reloading: Watch Timer / Manual Reload
    Reloading --> Loading: Reload Complete

    note right of Loading
        Running source command
        Streaming results to matcher
    end note

    note right of Filtering
        Real-time fuzzy matching
        UI updates every few ms
    end note
```

```mermaid
graph TB

    subgraph "Runtime Channel"
        F[Channel Instance] --> G[Matcher]
    end

    subgraph "Channel Operations"
        J[Load] --> K[Execute Source]
        K --> L[Stream Results]
        M[Filter & Match] --> N[Update UI]
        N --> O[Handle Selection]
    end

    L --> G
    G --> M
```

#### Channel Config (`channels/prototypes/`)

Channels are defined in TOML files:

```toml
[metadata]
name = "files"
description = "File finder"

[source]
command = "fd -t f"

[preview]
command = "bat --color=always '{}'"

[ui]
preview_panel = { size = 70 }

[keybindings]
shortcut = "f1"
```

#### Channel Runtime (`channels/channel.rs`)

- **Purpose:** Run source commands and manage results
- **Features:**
  - Async command execution with streaming results
  - Fuzzy matching with nucleo
  - Reload with debouncing
  - Multiple source commands

### Rendering System

#### Render Loop (`loops/render_loop.rs`)

- **Purpose:** Update the UI without blocking the main loop
- **Input:** Rendering tasks via channel
- **Output:** Terminal updates and UI state feedback
- **Features:**
  - 60 FPS frame rate capping to avoid CPU hogging
  - Synchronized terminal updates
  - Layout state tracking

#### Drawing (`draw.rs`)

- **Purpose:** Coordinate UI component rendering
- **Components:**
  - Input box with cursor
  - Results list with selection
  - Preview panel with content
  - Status bar with info
  - Remote control panel

### Configuration System

The configuration system uses a three-layer architecture implemented by the `LayeredConfig` struct, which merges configuration from multiple sources with proper priority ordering:

```mermaid
flowchart TD
    A[Base Config] --> C[LayeredConfig]
    B[Channel Prototype] --> C
    D[CLI Args] --> C
    C --> E[MergedConfig]

    subgraph "Config Sources"
        G[embedded config.toml] --> A
        I[$HOME/.config/television/config.toml] --> A
        J[cable/*.toml] --> B
        K[Command Line Args] --> D
    end

    subgraph "LayeredConfig Processing"
        C --> L[CLI-only fields]
        C --> M[Base config fields]
        C --> N[Channel-only fields]
        C --> O[Merged fields with priority]
    end

    subgraph "Priority Order (highest to lowest)"
        P[CLI Arguments] --> Q[Channel Configuration]
        Q --> R[User Configuration]
        R --> S[Embedded Defaults]
    end
```

**Key Components:**

- **Base Config**: User configuration loaded from `config.toml`
- **Channel Prototype**: Channel-specific configuration from cable files
- **CLI Configuration**: Runtime overrides from command-line arguments
- **MergedConfig**: Final resolved configuration used by the application

The layered approach allows for:
- Clean separation of concerns between different config sources
- Runtime channel switching without config reloading
- Proper inheritance and override behavior
- Type-safe configuration merging

### Preview System (`previewer/`)

```mermaid
sequenceDiagram
    participant UI as UI Component
    participant TV as Television
    participant PR as Previewer
    participant CMD as Preview Command

    UI->>TV: Selection Changed
    TV->>PR: Preview Request (entry)
    PR->>CMD: Execute preview command
    CMD-->>PR: Command output
    PR->>TV: Preview Response
    TV->>UI: Update preview panel

    Note over PR: Caching & Debouncing
    Note over CMD: Async execution
```

- **How it works:** Separate async task for non-blocking previews
- **Communication:** Request/response via channels
- **Features:**
  - Command-based preview generation
  - Caching and debouncing
  - Error handling and fallbacks
  - Syntax highlighting support

### Watch Timer (`loops/watch_timer.rs`)

- **Purpose:** Automatically reload channels
- **Features:**
  - Configurable intervals per channel
  - Auto start/stop on channel switch
  - Handles missed ticks

## Communication

```mermaid
flowchart LR
    subgraph "Action Flow"
    direction LR
        A[event_loop] --> B[action_channel] --> C[app_handler] --> D[television]
    end

    subgraph "Render Flow"
    direction LR
        E[app_handler] --> F[render_channel] --> G[render_loop] --> H[terminal]
    end

    subgraph "Preview Flow"
    direction LR
        I[television] --> J[preview_request] --> K[previewer] --> L[preview_response] --> I
    end
```

### Data Flow

- **One direction:** Events → Actions → State changes → Render
- **Feedback:** UI state info flows back for optimization
- **Async:** All blocking operations happen in separate tasks

## Design Patterns

### 1. Actor Model

Each major component runs independently and communicates via messages.

### 2. Command Pattern

All user interactions become Action enums.

### 3. Observer Pattern

UI state changes automatically trigger rendering updates.

### 4. Plugin Architecture

Channels are dynamically loaded from TOML config files.

## Performance

- **Event Processing:** Non-blocking with batched processing
- **Rendering:** Capped at 60 FPS with dirty state tracking
- **Matching:** Incremental fuzzy matching with nucleo
- **Preview:** Async with caching and debouncing
- **Memory:** Bounded result sets with efficient data structures

## How to Extend

### Adding New Channels

1. Create TOML config file
2. Define source command and output format
3. Add preview command and UI settings (optional)
4. Put in cable directory

### Custom Keybindings

- Global keybindings in main config
- Channel-specific keybindings in channel config
- Runtime updates via remote control

### UI Themes

- Color scheme definitions in theme files
- Component-specific styling
- Runtime theme switching

This architecture keeps things modular and fast, with clear separation between components and efficient async communication.
