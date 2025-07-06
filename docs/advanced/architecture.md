# Television Architecture

## Static System Architecture

### Context Diagram

```text
┌────────────────────────────────────────────────────────────────────────────┐
│                          Television Application                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│    ┌──────────────────┐    ┌────────────────┐    ┌───────────────────┐     │
│    │    CLI Module    │───►│ App Controller │◄───│ Config Management │     │
│    └──────────────────┘    └────────────────┘    └───────────────────┘     │
│                                     │                                      │
│                                     ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                           Television Core                            │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │   Channel   │  │   Picker    │  │  Previewer  │  │  Features   │  │  │
│  │  │  Management │  │ (UI State)  │  │   System    │  │ Management  │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                     │                                      │
│                                     ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                            Service Layer                             │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │   Matcher   │  │   History   │  │   Actions   │  │   Events    │  │  │
│  │  │   System    │  │ Management  │  │   System    │  │   Handler   │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                     │                                      │
│                                     ▼                                      │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                         Presentation Layer                           │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │  │
│  │  │   Screen    │  │    Draw     │  │   Render    │  │    TUI      │  │  │
│  │  │   Layout    │  │   System    │  │   Engine    │  │  Interface  │  │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Top Level Controllers

| Module | Description |
|--------|-------------|
| **CLI Module** | Parses command-line arguments and handles input validation and sanitization. |
| **App Controller** | Manages application lifecycle by orchestrating the main event loop and coordinating rendering. |
| **Config Management** | Coordinates configuration system with priority-based loading and validation. |

### Television Core

| Module | Description |
|--------|-------------|
| **Channel Management** | Loads channel prototypes and manages their execution lifecycle. Handles source command execution and result processing. |
| **Picker (UI State)** | Manages entry selection and filtering state. Tracks current selection and handles navigation logic. |
| **Previewer System** | Generates content previews asynchronously using template processing for dynamic content formatting. |
| **Features Management** | Controls UI component state including feature visibility and state transitions. |

### Service Layer

| Module | Description |
|--------|-------------|
| **Matcher System** | Implements fuzzy matching algorithms using `nucleo` for result ranking. |
| **History Management** | Tracks user interactions and maintains persistent search and selection history. |
| **Actions System** | Processes application actions and manages state transitions throughout the system. |
| **Events Handler** | Converts terminal events into application actions through input processing and event management. |

### Presentation Layer

| Module | Description |
|--------|-------------|
| **Screen Layout** | Manages ratatui constraint-based layouts and responsive design for terminal interfaces. |
| **Draw System** | Coordinates ratatui widgets by assembling components and handling state visualization. |
| **Render Engine** | Handles ratatui frame rendering with terminal buffer management and screen update. |
| **TUI Interface** | Manages crossterm terminal setup and captures events for cross-platform terminal control. |

## Dynamic System Architecture

### Core Data Flow

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                             Core Data Flow                              │
└─────────────────────────────────────────────────────────────────────────┘

  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌────────────┐
  │ User Input  │───►│    Event    │───►│ Television  │───►│ UI Render  │
  │ (Terminal)  │    │ Processing  │    │ Core Update │    │ (Terminal) │
  └─────────────┘    └─────────────┘    └─────────────┘    └────────────┘
                                               │
                                               ▼
                                      ┌──────────────────┐
                                      │     Channel      │
                                      │    Execution     │
                                      │ (Data Gathering) │
                                      └──────────────────┘
```

### Search and Matching Flow

```text
┌────────────────────────────────────────────────────────────────────────────────┐
│                            Search and Matching Flow                            │
└────────────────────────────────────────────────────────────────────────────────┘

     Input Pattern              Command Execution             Result Processing
    ┌─────────────┐          ┌──────────────────────┐          ┌─────────────┐
    │ User Types  │─────────►│       Channel        │─────────►│ Raw Entry   │
    │ Search Text │          │      Management      │          │ Data Stream │
    └─────────────┘          └──────────────────────┘          └─────────────┘
                                                                      │
                                                                      ▼
                             ┌──────────────────────┐          ┌─────────────┐
                             │ Matcher System       │◄─────────│ Entry List  │
                             │ - Fuzzy Matching     │          │ Processing  │
                             │ - Result Ranking     │          └─────────────┘
                             └──────────────────────┘
                                        │
                                        ▼
                             ┌──────────────────────┐
                             │ Filtered Results     │
                             │ - Matched Entries    │
                             │ - Ranking Scores     │
                             └──────────────────────┘
                                        │
                                        ▼
                             ┌──────────────────────┐
                             │ Picker (UI State)    │
                             │ - Selection State    │
                             │ - Display Formatting │
                             │ - Navigation Logic   │
                             └──────────────────────┘
```

### Event Processing Flow

```text
┌───────────────────────────────────────────────────────────────────────────┐
│                          Event Processing Flow                            │
└───────────────────────────────────────────────────────────────────────────┘

  Terminal Input             Event Processing               Action Dispatch
  ┌────────────┐          ┌─────────────────────┐          ┌───────────────┐
  │  Keyboard  │          │ Events Handler      │          │ Actions Queue │
  │   Events   │─────────►│ - Key Mapping       │─────────►│ - Input       │
  │            │          │ - Input Processing  │          │ - Navigate    │
  │   Mouse    │          │ - Event Filtering   │          │ - Select      │
  │   Events   │          └─────────────────────┘          │ - Toggle      │
  │            │                                           │ - Quit        │
  │  Terminal  │                                           └───────────────┘
  │   Events   │                                                  │
  └────────────┘                                                  ▼
                          ┌─────────────────────┐          ┌───────────────┐
                          │ Actions System      │          │  Television   │
                          │ - State Updates     │◄─────────│     Core      │
                          │ - Component Updates │          │    Update     │
                          │ - Render Triggers   │          └───────────────┘
                          └─────────────────────┘
                                     │
                                     ▼
                          ┌─────────────────────┐
                          │ UI Update           │
                          │ - Component Refresh │
                          │ - Screen Rendering  │
                          │ - State Propagation │
                          └─────────────────────┘
```

## Configuration System

```text
┌────────────────────────────────────────────────────────────────────────────┐
│                              Configuration System                          │
└────────────────────────────────────────────────────────────────────────────┘

  Configuration                Configuration                  Final
  Sources                         Merger                   Configuration
  ┌─────────────┐          ┌───────────────────┐          ┌─────────────┐
  │    CLI      │─────────►│ Priority Order:   │          │   Merged    │
  │   Flags     │          │ 1. CLI Flags      │─────────►│ Application │
  └─────────────┘          │ 2. Channel Config │          │  Settings   │
                           │ 3. User Config    │          └─────────────┘
  ┌─────────────┐          │ 4. Built-in       │                 │
  │   Channel   │─────────►│    Defaults       │                 ▼
  │   Config    │          └───────────────────┘          ┌─────────────┐
  └─────────────┘                 ▲     ▲                 │ Television  │
                                  │     │                 │    Core     │
  ┌─────────────┐                 │     │                 └─────────────┘
  │ User Config │─────────────────┘     │
  │    File     │                       │
  └─────────────┘                       │
                                        │
  ┌─────────────┐                       │
  │  Built-in   │───────────────────────┘
  │  Defaults   │
  └─────────────┘
```

## Channel System

```text
┌───────────────────────────────────────────────────────────────────────────────────┐
│                                    Channel System                                 │
└───────────────────────────────────────────────────────────────────────────────────┘

   Channel Discovery             Channel Loading                 Channel Execution
    ┌─────────────┐          ┌─────────────────────┐              ┌─────────────┐
    │    Cable    │          │  Channel Prototype  │              │   Source    │
    │  Directory  │─────────►│ - Metadata          │─────────────►│   Command   │
    │    Scan     │          │ - Source Config     │              │  Execution  │
    └─────────────┘          │ - Preview Config    │              └─────────────┘
                             │ - UI Configuration  │                     │
    ┌─────────────┐          └─────────────────────┘                     ▼
    │   Ad-hoc    │              ▲      │                         ┌─────────────┐
    │  Channels   │──────────────┘      │                         │   Entry     │
    └─────────────┘                     ▼                         │ Processing  │
                             ┌─────────────────────┐              └─────────────┘
                             │ Channel Instance    │                     │
                             │ - State Management  │                     ▼
                             │ - Result Tracking   │              ┌─────────────┐
                             │ - Selection State   │◄─────────────│   Matcher   │
                             │ - Active Commands   │              │ Integration │
                             └─────────────────────┘              └─────────────┘
```

