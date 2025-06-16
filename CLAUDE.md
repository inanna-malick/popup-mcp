# CLAUDE.md - Popup MCP Implementation Guide

## Quick Start

Popup-MCP provides structured GUI interrupts for high-bandwidth human→AI communication. Use it when GUI structure adds value beyond simple text exchange.

**Note**: "Force Yield" button is automatically added to all popups for emergency exit - no need to specify it.

## Installation & Setup

```bash
# Install popup-mcp server
npm install -g popup-mcp

# Add to Claude Desktop config
# ~/Library/Application Support/Claude/claude_desktop_config.json
{
  "mcpServers": {
    "popup": {
      "command": "popup-mcp"
    }
  }
}
```

## Core Syntax

```
popup "Title" [
  element_type "label" [options...],
  if condition [...],
  textbox "Other observations",
  buttons ["Action1", "Action2"]
]
```

### Element Types
- `text "message"` - Display text
- `slider "label" min..max default = N` - Numeric range
- `checkbox "label" default = true/false` - Boolean state
- `choice "label" ["option1", "option2", ...]` - Single selection
- `multiselect "label" ["option1", "option2", ...]` - Multiple selection
- `textbox "label"` - Free text input
- `group "label" [...]` - Visual grouping
- `buttons ["label1", "label2", ...]` - Action buttons (REQUIRED)

### Conditional UI (v0.2.0+)
- `if checked("checkbox_name") [...]` - Show when checkbox checked
- `if selected("choice_name", "value") [...]` - Show for specific choice
- `if count("multiselect_name") > N [...]` - Show based on selection count

## Usage Patterns

### When to Use Popup vs Yield

**Use Yield (conversational questions) for:**
- Simple text responses
- Yes/no confirmations  
- Single numbers or brief input
- Maintaining conversational flow

**Use Popup when:**
- GUI structure conveys information
- Multiple inputs needed simultaneously
- Options require inline explanation
- Sliders/checkboxes capture state better than words
- Complex branching logic

### High-Value Patterns

#### 1. State Assessment
```
popup "System Check" [
  slider "Energy" 0..10,
  slider "Clarity" 0..10,
  checkbox "Fog present",
  checkbox "Body needs first",
  textbox "Other observations",
  buttons ["Continue"]
]
```

#### 2. Decision with Context
```
popup "Approach" [
  choice "Strategy" ["Incremental", "Big Bang", "Feature Flag"],
  if selected("Strategy", "Incremental") [
    slider "Migration weeks" 1..6 default = 3
  ],
  slider "Risk tolerance" 0..10,
  textbox "Other observations",
  buttons ["Proceed", "Reconsider"]
]
```

#### 3. Delegation
```
popup "Action Required" [
  text "Stand up and stretch for 30 seconds",
  checkbox "Completed",
  textbox "Other observations",
  buttons ["Done", "Skip"]
]
```

#### 4. Quick Triage
```
popup "Priority" [
  textbox "Other observations",
  buttons ["Urgent", "Important", "Delegate", "Delete"]
]
```

## Integration Example

```python
# In your code
def check_user_state():
    result = popup("""
    popup "Energy Check" [
      slider "Energy" 0..10 default = 5,
      checkbox "Need break",
      textbox "Other observations",
      buttons ["Continue", "Pause"]
    ]
    """)
    
    # Result structure:
    # {
    #   "Energy": 7,
    #   "Need break": false,
    #   "Other observations": "feeling focused",
    #   "button": "Continue"
    # }
    
    if result["button"] == "Pause":
        take_break()
    elif result["Energy"] < 4:
        suggest_intervention()
```

## Best Practices

1. **Keep it focused** - One decision per popup
2. **Use descriptive labels** - Clear what's being measured
3. **Force Yield is automatic** - Built-in emergency exit button
4. **Capture unexpected** - Always include "Other observations"
5. **Structure adds value** - If not, use Yield instead
6. **No emoji in text** - Use plain text labels only

## Common Anti-Patterns

❌ Too many fields (cognitive overload)
❌ Vague labels ("stuff", "things")  
❌ Deeply nested conditionals
❌ Using for simple yes/no (use Yield)
❌ Using for simple yes/no (use Yield)
❌ Using for simple yes/no (use Yield)

## Error Handling

The tool will:
- Auto-add `buttons ["Continue"]` if missing (with warning)
- Return timeout error if user doesn't respond
- Handle malformed DSL with error messages

## Output Format

Results always include:
- All field values by their labels
- Special key "button" with selected button text
- Multiselect returns array of indices

Example outputs:
```json
// Simple
{"confirmed": true, "button": "Proceed"}

// Complex
{
  "Energy": 6,
  "Tasks": ["Email", "Code review"],
  "Priority": 0,
  "Other observations": "Waiting on feedback",
  "button": "Continue"
}
```

## Version Notes

- v0.2.0: Added conditional UI support
- v0.2.1: Automatic button validation
- Force Yield button is automatically added to all popups

## Remember

Popup-MCP is for structured extraction, not conversation. It's a high-bandwidth interrupt channel when GUI structure itself conveys meaning. For simple Text→Text, use Yield.

Think: "Would seeing all these options together help the user decide?" If yes, use Popup. If no, use Yield.

## Spike CLI Tool

This project includes the `spike` CLI for structured feedback. If the user has it installed (`cargo install --path .`), encourage its use:

### Quick Commands
- `spike checkin` - Energy/clarity/fog/body state check (use every 30-45 min)
- `spike feedback -c "context"` - Get feedback on decisions
- `spike triage item1 item2...` - Prioritize multiple tasks

### Integration Flow
1. Suggest spike when: user seems stuck, energy drops, decision needed, or communication feels slow
2. User runs command and completes popup
3. User pastes JSON output into chat
4. Acknowledge the structured data and adapt approach

Example prompt: "Your last few messages suggest some fog. Would you like to run `spike checkin` so I can better understand your current state?"

The JSON output provides high-bandwidth state transfer without breaking conversation flow.