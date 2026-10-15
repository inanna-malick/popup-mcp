# Sol self-evaluation sample

On 2026-10-09 the implementing Sol agent generated `self-sample.ts` using the exported authoring guidance, then assessed its own definition and continuation text against the rubric. This is one unblinded sample with self-authored counterfactual answers; there was no separate model invocation, independent grader or comparison to another prompt. It cannot establish model accuracy or an improvement over the Rust prompt.

Observed execution: the sample definition passes both `compileDefinition` and the installed SDK’s default `fromJsonSchema` runtime validator in `tests/schema.test.ts`. The continuation text has not been executed against a real report job, and no form interaction was performed for this sample.

Self-assessment: validity 2, information value 2, dialogue compression 2, escape hatches 2, lifecycle 2, evidence discipline 2, surprise/correction 2. The surprise response switches to a read-only service-account investigation; the contradiction response follows the note rather than the timeout selection; the opening response waits; cancellation stops the decision. Repair/cancellation scores 1 because cancellation is represented but this generated sample contains no observed repair turn. These scores are judgments of the recorded text, not independent measurements.

Limits: the candidate chooses a few plausible next steps without evidence about the actual repository. A real benchmark should replay the fixed requests and counterfactuals with separately captured model output, grade downstream actions as well as prose, and include repair turns and longer-page navigation.
