# Popup-MCP: Structured GUI for AI Communication

Popup-MCP provides structured GUI interrupts for high-bandwidth human→AI communication. Use it when GUI structure adds value beyond simple text exchange.

## Quick Setup

```bash
# Install globally
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

## Core Syntax & Elements

```
popup "Title" [
  element_type "label" [options...],
  if condition [...],
  buttons ["Action1", "Action2"]
]
```

### Available Elements
- `text "message"` - Display text
- `slider "label" min..max default=N` - Numeric range
- `checkbox "label" default=true/false` - Boolean toggle
- `choice "label" ["option1", "option2"]` - Single selection
- `multiselect "label" ["option1", "option2"]` - Multiple selection
- `textbox "label"` - Free text input
- `group "label" [...]` - Visual grouping
- `buttons ["label1", "label2"]` - Action buttons (required)

### Conditional UI
- `if checked("checkbox_name") [...]` - Show when checkbox checked
- `if selected("choice_name", "value") [...]` - Show for specific choice
- `if count("multiselect_name") > N [...]` - Show based on selection count

## When to Use Popups

**Use conversational prompts for:**
- Simple text responses
- Yes/no confirmations
- Single values
- Maintaining flow

**Use Popup when:**
- Multiple inputs needed simultaneously
- GUI structure conveys information
- Visual elements (sliders, checkboxes) communicate better than words
- Complex branching logic required

## Examples

### State Assessment
```
popup "System Check" [
  slider "Energy" 0..10,
  slider "Focus" 0..10,
  checkbox "Needs break",
  textbox "Notes",
  buttons ["Continue", "Pause"]
]
```

### Contextual Decision
```
popup "Migration Strategy" [
  choice "Approach" ["Incremental", "Big Bang", "Feature Flag"],
  if selected("Approach", "Incremental") [
    slider "Phases" 1..6 default=3
  ],
  slider "Risk tolerance" 0..10,
  buttons ["Proceed", "Reconsider"]
]
```

## Integration

```python
def check_user_state():
    result = popup("""
    popup "Energy Check" [
      slider "Energy" 0..10 default=5,
      checkbox "Need break",
      buttons ["Continue", "Pause"]
    ]
    """)
    
    # Result: {"Energy": 7, "Need break": false, "button": "Continue"}
    
    if result["button"] == "Pause":
        take_break()
    elif result["Energy"] < 4:
        suggest_intervention()
```

## Best Practices

1. **One decision per popup** - Keep focused
2. **Descriptive labels** - Be specific about what's measured
3. **Include escape hatch** - "Force Yield" button is automatic
4. **Capture edge cases** - Add "Other" textbox when appropriate
5. **Structure must add value** - Otherwise use conversational prompts

## Technical Notes

### Output Format
Results return all field values by label, plus a special "button" key:

```json
{
  "Energy": 7,
  "Tasks": ["Email", "Code review"],
  "Notes": "Waiting on feedback",
  "button": "Continue"
}
```

### Error Handling
- Missing buttons trigger auto-add with warning
- Malformed DSL returns descriptive errors
- User timeouts handled gracefully

## Key Insight

Popup-MCP is for structured extraction, not conversation. It's a high-bandwidth channel when GUI structure itself conveys meaning. Ask yourself: "Would seeing these options together help the user decide?" If yes, use Popup. If no, use conversational prompts.