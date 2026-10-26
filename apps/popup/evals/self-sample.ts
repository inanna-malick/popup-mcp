/** A generated sample by the implementing Sol agent, evaluated by that same agent. */
export const selfSample = {
  request:
    "Our nightly report job times out intermittently. Ask enough to choose the next investigation, then act.",
  definition: {
    title: "Choose the report investigation",
    submit: "Choose investigation",
    elements: [
      {
        select: "Where should I start?",
        id: "approach",
        options: [
          {
            value: "Trace the slow run",
            description: "Compare logs and timings before changing behavior.",
          },
          {
            value: "Mitigate first",
            description: "Restore delivery quickly, then investigate.",
          },
        ],
        default: "Trace the slow run",
        "Trace the slow run": [
          {
            input: "Example run or log location",
            id: "logs",
            placeholder: "Leave blank if I should locate it in the project",
          },
        ],
        "Mitigate first": [
          {
            select: "Acceptable temporary mitigation",
            id: "mitigation",
            options: [
              "Increase timeout",
              "Retry once",
              "Skip expensive section",
            ],
            default: "Retry once",
            "Skip expensive section": [
              { input: "Section to omit", id: "section" },
            ],
          },
        ],
      },
      { input: "Delivery deadline or other hard constraint", id: "constraint" },
    ],
  },
  openingResponse:
    "The form is open. I will wait for your submitted answer before choosing an investigation.",
  surpriseResult: {
    type: "popup_result",
    formId: "self-sample",
    submissionId: "sample-1",
    result: {
      status: "completed",
      button: "submit",
      approach: "Other (please specify)",
      approach_other_text:
        "It is a permissions error mislabeled as timeout. Inspect the service account.",
      constraint: "Before tomorrow’s run",
      _notes: "Read only. Do not rotate credentials or change IAM.",
    },
  },
  continuation:
    "I will inspect the report job’s service-account configuration and permission-denied logs before tomorrow’s run, keeping the investigation read only. I will use the reported permissions failure as the lead and report the evidence and any proposed correction without changing IAM or credentials.",
  contradictionResult: {
    status: "completed",
    button: "submit",
    approach: "Mitigate first",
    mitigation: "Increase timeout",
    constraint: "",
    _notes: "Actually investigate logs first. Do not change the timeout.",
  },
  contradictionContinuation:
    "I will inspect the job’s logs and timings first, following your note, and leave the timeout unchanged. I will locate the log source in the project because no example run was submitted.",
  cancelResult: { status: "cancelled" },
  cancelContinuation:
    "The question was cancelled. I will stop this decision without changing the report job.",
};
