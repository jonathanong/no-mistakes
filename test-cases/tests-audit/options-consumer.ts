import type { TestsAuditOptions } from "../../packages/no-mistakes/test-audit-types";

const files: TestsAuditOptions = { plan: "plan.json", observations: "run.json" };
const inline: TestsAuditOptions = { planJson: "{}", observationsJson: "{}" };
const mixed: TestsAuditOptions = { plan: "plan.json", observationsJson: "{}" };

// @ts-expect-error Both artifacts are required.
const empty: TestsAuditOptions = {};
// @ts-expect-error An observations input is required.
const missing: TestsAuditOptions = { plan: "plan.json" };
// @ts-expect-error A plan cannot have two input forms.
const conflictingPlan: TestsAuditOptions = {
  plan: "plan.json",
  planJson: "{}",
  observations: "run.json",
};
// @ts-expect-error Observations cannot have two input forms.
const conflictingRun: TestsAuditOptions = {
  plan: "plan.json",
  observations: "run.json",
  observationsJson: "{}",
};

void [files, inline, mixed, empty, missing, conflictingPlan, conflictingRun];
