# Spike CLI

A command-line tool for structured feedback collection during Claude Code sessions.

## Installation

```bash
cargo install --path .
# or for development
cargo build --release
cp target/release/spike ~/.cargo/bin/
```

## Usage

### Default (General Feedback)
```bash
spike
```
Shows a general-purpose feedback popup.

### Check-in
```bash
spike checkin
```
Quick system check for energy, clarity, fog, and body needs.

### Feedback
```bash
spike feedback --context "working on authentication"
```
Get structured feedback on a specific approach or decision.

### Triage
```bash
spike triage "fix bug" "write tests" "update docs"
```
Quick priority triage for multiple tasks.

### Show Popup File
```bash
spike show examples/my_custom.popup
```
Display any .popup file.

### Run Inline DSL
```bash
spike run 'popup "Quick Test" [text "Hello!", buttons ["OK"]]'
```
Run popup DSL directly from command line.

## Output

All commands output JSON to stdout for easy parsing:

```json
{
  "Energy": 7,
  "Clarity": 8,
  "Fog present": false,
  "Body needs first": false,
  "Other observations": "Feeling focused",
  "button": "Continue"
}
```

## Integration with Claude Code

During a Claude Code session, you can use spike to:

1. **Regular check-ins**: `spike checkin` to communicate your current state
2. **Decision points**: `spike feedback -c "about to refactor the API"` 
3. **Task prioritization**: `spike triage "task1" "task2" "task3"`
4. **Custom feedback**: Create .popup files for session-specific needs

The JSON output can be easily copied into the Claude conversation for structured communication.