
  Complex Nested Decision Tree Scenario

  Production Incident Diagnosis Assistant

  Context: On-call engineer is investigating a production incident. The assistant needs to quickly narrow down root cause by asking adaptive questions based on previous answers.
  The decision tree should reveal itself progressively - initial questions are broad, then become increasingly specific based on what's selected.

  Complexity requirements:
  - 3-4 levels of nesting minimum
  - Parent-state conditionals (most common)
  - Cross-element conditionals with boolean logic
  - Count-based logic (multiselect)
  - Slider threshold checks
  - Multiple branching paths from initial choice

  User journey:

  Level 0: Initial Classification

  User selects incident type: Performance, Error Rate, or Data Integrity

  Level 1: Performance Path

  If Performance selected:

  Temporal pattern question: When did degradation start?
  - Sudden (after deployment)
  - Gradual (over days)
  - Intermittent (no pattern)

  Level 2a: Sudden Degradation Branch

  If Sudden selected:

  Change detection: What changed in last deployment? (multiselect)
  - New code
  - Configuration changes
  - Database schema migration
  - Infrastructure scaling
  - Dependency updates

  Conditional logic at this level:
  - If count(changes) >= 3: Show warning text "Multiple simultaneous changes detected. High complexity deployment - recommend immediate rollback."
  - If count(changes) == 1: Show text "Single change isolation - good signal for root cause"

  Level 3a: Code Change Sub-branch

  If "New code" selected from changes:

  Service isolation: Which services were deployed? (multiselect with options)
  - Auth service
  - API gateway
  - Database layer
  - Cache layer
  - Background workers

  Deep conditional:
  - If count(services) == 1: Show text "✓ Isolated deployment" + Show textbox "What does this service do?"
  - If count(services) >= 3: Show text "⚠ Broad deployment" + Show checkbox "Can you isolate to one service?"
    - If that checkbox checked: Show dropdown "Which service shows symptoms?"

  Level 3b: Configuration Change Sub-branch

  If "Configuration changes" selected:

  Config type: What configuration changed?
  - Connection pool size
  - Timeout values
  - Cache TTL
  - Rate limits

  Each option has conditionals:
  - If "Connection pool size" selected: Show slider "New pool size" (1-100)
    - If slider > 50: Show text "Large pool - check connection exhaustion"
    - If slider < 10: Show text "Small pool - check queueing"

  Level 2b: Resource Metrics (Parallel to temporal)

  System health check: Current resource utilization (always visible after Performance selected)

  Sliders for:
  - CPU usage % (0-100)
  - Memory usage % (0-100)
  - Disk I/O % (0-100)
  - Network bandwidth % (0-100)

  Cross-slider conditionals:
  - If CPU > 80 && Memory > 80: Show critical alert text + Show checkbox "Enable emergency rate limiting?"
  - If Disk I/O > 90 && (count(changes with "Database schema migration") > 0): Show text "Correlation: Schema migration + disk saturation = missing index?"
  - If all metrics < 50: Show text "Resources healthy - check application-level bottleneck"

  Level 2c: Gradual Degradation Branch

  If Gradual selected (back at Level 1):

  Growth metrics: What's been trending upward? (multiselect)
  - Request volume
  - Database size
  - Active user count
  - Cache miss rate

  Per-metric conditionals:
  - If "Request volume" selected: Show slider "Traffic increase %" (0-500)
    - If slider > 200: Show text "High growth - capacity planning issue"
    - Show choice "Traffic pattern: Organic / Bot traffic / Viral event"
        - If "Bot traffic" selected: Show textbox "Bot user agent patterns observed?"
  - If "Database size" selected: Show textbox "Which tables growing fastest?"
    - Show checkbox "Have you checked for data retention policies?"
  - If count(growth metrics) >= 3: Show text "Multiple growth vectors - systemic scale issue, not isolated problem"

  Level 3: Cross-path conditionals

  Hypothesis ranking section (shown after any branch provides enough data):

  - If any of (Sudden selected, count(changes) > 0, CPU > 80): Show slider "Deployment issue likelihood" (0-100)
  - If any of (Gradual selected, Request volume selected, Database size selected): Show slider "Capacity issue likelihood" (0-100)
  - If any of (Intermittent selected, Disk I/O > 90): Show slider "Infrastructure issue likelihood" (0-100)

  Final synthesis:
  - If (Deployment issue > 70) && (Capacity issue < 30): Show text "Strong signal: Recent change caused issue. Recommend rollback."
  - If (Capacity issue > 70) && (Deployment issue < 30): Show text "Strong signal: Organic growth exceeded capacity. Scale infrastructure."
  - If (Deployment issue > 50) && (Capacity issue > 50): Show text "Mixed signal: Both factors present. Prioritize rollback, then scale."
