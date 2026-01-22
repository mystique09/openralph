# OpenRalph

A tiny "Ralph Wiggum" loop harness for OpenCode: run an agent repeatedly until it outputs a completion phrase (default: `DONE`) or you hit a max-iteration safety limit.

This tool runs `opencode run ...` in a loop and captures stdout/stderr so it can detect the completion phrase reliably.

## Overview

OpenRalph is a Rust CLI tool that automates iterative agent workflows. It wraps the OpenCode agent runner and provides:

- **Loop control**: Run agents for multiple iterations until completion
- **Safety limits**: Maximum iteration count to prevent infinite loops
- **Completion detection**: Flexible phrase matching to know when work is done
- **State management**: Combine agent instructions and project plans into prompts
- **Output capture**: Capture both stdout and stderr for reliable completion detection

Perfect for multi-step development tasks, automated testing workflows, or any scenario where an agent needs to work through a series of tasks sequentially.

## Installation

### From Source

```bash
# Clone the repository
git clone <repository-url>
cd open-ralph

# Build and install
cargo install --path .
```

### Using Cargo Directly

If you have the source code:

```bash
cargo build --release
```

The compiled binary will be available at `target/release/openralph`.

### Dependencies

OpenRalph requires:
- Rust edition 2024 or later
- `opencode` CLI tool must be available in your PATH

## Usage

### Basic Example

```bash
openralph --prompt plan.md
```

This runs the agent with the default settings:
- Max iterations: 10
- Completion phrase: `DONE`
- Sleep between iterations: 2 seconds

### Common Patterns

```bash
# Run up to 10 iterations
openralph --prompt plan.md --max-iterations 10

# Use a custom completion phrase
openralph --prompt plan.md --completion "ALL_TASKS_COMPLETE"

# Increase sleep time between iterations (useful for rate limiting)
openralph --prompt plan.md --sleep-secs 5

# Combine multiple options
openralph --prompt tasks.json --max-iterations 20 --completion "✅" --sleep-secs 3
```

## CLI Arguments

| Argument | Short | Type | Default | Description |
|----------|-------|------|---------|-------------|
| `--prompt` | `-p` | Path | Required | Path to the plan file containing task definitions |
| `--max-iterations` | `-n` | Number | `1` | Maximum number of iterations to run (safety limit) |
| `--completion` | `-c` | String | `"DONE"` | Completion phrase to detect in agent output |
| `--sleep-secs` | `-s` | Number | `2` | Seconds to sleep between iterations |

### Argument Details

#### `--prompt` (required)

The plan file contains your task definitions in JSON format. OpenRalph reads this file and combines it with the built-in `instructions.md` to create the full prompt for the OpenCode agent.

Example:
```bash
openralph --prompt project-plan.md
```

#### `--max-iterations`

Safety limit to prevent infinite loops. The agent will stop after this many iterations regardless of whether the completion phrase was detected.

**Example:**
```bash
# Allow up to 50 iterations
openralph --prompt plan.md --max-iterations 50
```

**Best Practice:** Set this higher than you expect to need, but not so high that you risk runaway agent loops.

#### `--completion`

The phrase that OpenRalph searches for in the agent's combined stdout/stderr output to determine if the work is complete. When detected, the loop ends immediately and the tool reports success.

**Examples:**
```bash
# Default
openralph --prompt plan.md --completion "DONE"

# Custom phrases
openralph --prompt plan.md --completion "ALL TASKS COMPLETE"
openralph --prompt plan.md --completion "🎉"
openralph --prompt plan.md --completion "FINISHED"
```

**Important:** The completion phrase must appear **exactly** as specified (case-sensitive match).

#### `--sleep-secs`

Delay between iterations. Useful for:
- Preventing overwhelming system resources
- Adding cooldown time for dependent systems

**Example:**
```bash
# Wait 5 seconds between iterations
openralph --prompt plan.md --sleep-secs 5
```

## How It Works

### Execution Flow

```
1. Load plan.md and instructions.md
2. For each iteration (1 to max_iterations):
   ├─ Read current plan state
   ├─ Build prompt: instructions + plan + completion-text
   ├─ Run: opencode run <prompt>
   ├─ Capture stdout and stderr
   ├─ Print output to console
   ├─ Check if completion phrase detected
   │  └─ If found: Mark complete, exit loop
   ├─ Check if opencode exited with error
   │  └─ If error: Log warning, continue
   └─ Sleep for sleep-secs
3. Report final status
```

This means the completion phrase can appear in either stream, giving flexibility to agent developers.

### Error Handling

- If `opencode` exits with a non-zero code, OpenRalph logs a warning but **continues** to the next iteration
- If the completion phrase is never detected and `max_iterations` is reached, OpenRalph reports: "Reached max iterations without seeing completion phrase"
- Plan file read errors or other I/O issues cause OpenRalph to exit immediately with an error

## Configuration

### Plan File Format (`plan.md`)

The plan file contains a JSON array of task definitions:

```json
[
  {
    "category": "project",
    "description": "Create a new rust project that logs 'Hi.'",
    "steps": [
        "Initialize a new rust project using cargo name the new rust project example/.",
        "Modify main.rs file of the new rust project"
    ],
    "passes": true
  },
  {
    "category": "feature",
    "description": "Create a add(a, b) function that adds two number.",
    "steps": [
        "Inside the example/",
        "Create a function",
        "Invoke in main function",
        "Log the output"
    ],
    "passes": false
  }
]
```

**Fields:**
- `category`: Grouping for the task (e.g., "project", "feature", "bugfix")
- `description`: What the task should accomplish
- `steps`: Array of implementation steps
- `passes`: Boolean flag indicating if the task is complete

### Instructions File (`instructions.md`)

OpenRalph includes a built-in `instructions.md` that tells the agent how to interact with the plan:

1. Read activity.md to understand current state
2. Select the next task where `passes: false` (only one at a time)
3. Implement only that task
4. Run verification commands (e.g., `cargo check`) and fix until they pass
5. Update only that task's `passes` field to `true` (do not edit task text/ordering)
6. Append a log entry to activity.md
7. If all tasks have `passes: true`, output exactly `<completion-text>` and nothing else

### Activity Logging

The agent maintains an `activity.md` file that tracks progress:

```markdown
# Project Build - Activity Log

## Current Status
**Last Updated:** 2025-01-23 10:30:00
**Tasks Completed:** 2/5
**Current Task:** Create a add(a, b) function

---

## Session Log

2025-01-23 10:00:00 - Task 1 completed: Created new Rust project
2025-01-23 10:15:00 - Task 2 completed: Added add() function, cargo check passed
```

## Examples

### Simple Development Workflow

```bash
# Create a plan file for a Rust library
cat > lib-plan.md << 'EOF'
[
  {
    "category": "project",
    "description": "Initialize a new Rust library project",
    "steps": [
      "Run cargo init --lib",
      "Update Cargo.toml with library metadata"
    ],
    "passes": false
  },
  {
    "category": "feature",
    "description": "Add a greet(name) function",
    "steps": [
      "Implement greet function in src/lib.rs",
      "Add documentation comments",
      "Run cargo test to verify"
    ],
    "passes": false
  },
  {
    "category": "feature",
    "description": "Add unit tests for greet function",
    "steps": [
      "Add #[cfg(test)] module in src/lib.rs",
      "Write tests for edge cases",
      "Run cargo test to verify all pass"
    ],
    "passes": false
  }
]
EOF

# Run OpenRalph with up to 20 iterations
openralph --prompt lib-plan.md --max-iterations 20
```


## Contributing

Contributions are welcome! This is a small, focused tool, please keep PRs concise and well-documented.

### Development

```bash
# Run tests
cargo test

# Run with debug output
cargo run -- --prompt plan.md --max-iterations 5

# Format code
cargo fmt

# Run clippy
cargo clippy
```

## Acknowledgments

- Named after Ralph Wiggum from The Simpsons
- [Running Ralph Wiggum the Right Way: A Complete Setup Guide](https://github.com/JeredBlu/guides/blob/main/Ralph_Wiggum_Guide.md)
- Built with [clap](https://github.com/clap-rs/clap) for CLI parsing
