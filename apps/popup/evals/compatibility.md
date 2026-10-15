# Repository example compatibility

Observed locally on 2026-10-09 using `npm test -- tests/schema.test.ts`: 22 of the 23 repository `examples/*.json` files compile with the classic TypeScript parser and pass the MCP SDK 2.0.0 `fromJsonSchema` default runtime validator.

The excluded `incident_triage_tree.json` is a semantic-routing example using `page.next`, outside this app’s intentional linear-page scope. Both validators reject it. This is not a claim that every historical Rust extension is supported.

Accepted: api_performance_demo, api_performance_enhanced, api_performance_full, api_performance_smart, audio_mastering, choice_demo, cloud_infrastructure, conditional_settings, feedback_form, fix_strategy_interview, k8s_manifest_interview, philosophical_framework, project_setup_wizard, research_methodology, rpg_worldbuilding_magitech, settings, simple_confirm, state_checkin_v2, story_builder, system_status, unified_conditionals and user_profile (all `.json`).

The same test checks published examples, offline reference fixtures, alias conflicts, rows bounds and unsupported routing. An in-memory MCP client/server verifies that tools/list publishes the exact recursive schema, including its descriptions and examples. Structural schema validation is followed by parser checks: cross-field slider bounds, option uniqueness, ids, dependency cycles and total node/depth/payload limits require semantic validation beyond ordinary JSON Schema constraints.
