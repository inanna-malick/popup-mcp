# Spike CLI Integration

This codebase includes the `spike` CLI tool for structured feedback during our session. Use it to communicate your state, get feedback, or make decisions.

## Available Commands

### Quick Check-in
```bash
spike checkin
```
Use this periodically to communicate your energy, clarity, and needs. I'll see output like:
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

### Get Feedback
```bash
spike feedback --context "what you're working on"
```
Use when you want structured feedback on an approach or decision.

### Triage Tasks
```bash
spike triage "task1" "task2" "task3"
```
Use when you need to prioritize multiple items.

### Custom Popups
```bash
spike show path/to/custom.popup
```
Use for project-specific structured inputs.

## When to Use Spike

- **Every 30-45 minutes**: Run `spike checkin` to update me on your state
- **At decision points**: Use `spike feedback -c "context"` before major changes
- **When overwhelmed**: Use `spike triage` to prioritize tasks
- **When communication feels inefficient**: Create a custom .popup file for the specific structure you need

## Integration Protocol

1. Run the spike command in your terminal
2. Complete the popup interface
3. Copy the JSON output
4. Paste it into our conversation
5. I'll acknowledge and adapt based on your feedback

Example:
```
You: *runs spike checkin*
You: ```json
{
  "Energy": 4,
  "Clarity": 6,
  "Fog present": true,
  "Body needs first": false,
  "Other observations": "Getting tired, might need to simplify approach",
  "button": "Continue"
}
```
Me: I see your energy is at 4 and fog is present. Let's simplify our approach...
```

This creates a structured interrupt channel that respects your cognitive state while maintaining conversation flow.