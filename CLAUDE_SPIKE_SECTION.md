# Spike CLI for Coding Sessions

The `spike` CLI provides structured GUI interrupts for high-bandwidth decision making and state management during coding sessions. Use it when you need more nuanced input than a simple text response.

## Available Commands

### Architecture Decisions
```bash
spike feedback --context "choosing between REST and GraphQL"
```
Get structured feedback on technical decisions with sliders for factors like complexity, performance needs, team familiarity.

### Code Review State
```bash
spike review
```
Communicate your review progress and findings:
- Severity sliders for different issue types
- Checkboxes for common problems found
- Quick categorization of changes

### Refactoring Scope
```bash
spike refactor --context "legacy auth system"
```
Define refactoring boundaries:
- Risk tolerance slider
- Time estimate range
- Affected systems checklist
- Migration strategy choice

### Debug Session State
```bash
spike debug
```
Track debugging progress:
- Hypothesis confidence levels
- Areas investigated checklist
- Reproduction success rate
- Next steps prioritization

### Task Prioritization
```bash
spike triage "fix auth bug" "add logging" "update docs" "review PR"
```
Quick drag-and-drop or button-based prioritization when juggling multiple tasks.

## Integration Protocol

1. I'll suggest spike when I detect:
   - Complex architectural decisions
   - Multiple implementation paths
   - Unclear requirements
   - Debugging dead ends
   - Task overload

2. Run the command and complete the popup
3. Paste the JSON output back
4. I'll use the structured data to provide better guidance

## Example Use Cases

### Implementation Strategy
```
You: "Should I refactor this 500-line function?"
Me: "Let's get more context. Could you run `spike refactor --context 'process_order function'`?"
You: *pastes JSON with risk=3, time=8hrs, systems=["orders", "inventory"], strategy="incremental"*
Me: "With low risk tolerance and 8 hours available, let's do incremental extraction starting with the inventory updates..."
```

### Debugging Dead End
```
Me: "We've tried three hypotheses without success. Run `spike debug` to help me understand what you're observing."
You: *pastes JSON showing confidence levels and investigated areas*
Me: "I see you have high confidence in the data layer but haven't checked the event handlers. Let's look there next..."
```

### Feature Planning
```
You: "Need to add real-time notifications"
Me: "Run `spike feedback --context 'real-time notifications'` so I can understand your constraints."
You: *pastes JSON with latency=100ms, scale=1000users, complexity_ok=7*
Me: "With those parameters, WebSockets would work well. Here's an implementation plan..."
```

## Custom Popups for Your Project

Create project-specific popups in `~/.spike/`:

```
# ~/.spike/pr_review.popup
popup "PR Review Helper" [
  slider "Code quality" 1..10,
  multiselect "Issues found" [
    "Missing tests",
    "No error handling", 
    "Performance concerns",
    "Security issues",
    "Style inconsistencies"
  ],
  checkbox "Needs discussion",
  textbox "Other observations",
  buttons ["Approve", "Request changes", "Needs work", "Skip"]
]
```

Then use: `spike show pr_review`

## Why This Helps

- **Parallel Input**: Communicate multiple factors simultaneously
- **Precision**: Sliders and ranges are clearer than "somewhat" or "pretty much"
- **Context Preservation**: Structured data maintains decision context
- **Cognitive Offload**: Externalize complex state into GUI
- **Faster Iteration**: Skip back-and-forth clarification

## Remember

Spike is for extracting nuanced, multi-dimensional input when text would be cumbersome. It's not for simple yes/no questions - just type those. Think: "Would seeing all these options together help me communicate my constraints/observations better?"