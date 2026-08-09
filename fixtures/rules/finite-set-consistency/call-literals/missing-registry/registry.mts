export const AI_AGENTS_SCHEDULED_JOBS = [
  { id: "reconcileAutoDispatchJudgements" },
  // Regression guard: reconcileRuntimeGenerations is registered in schedules.mts but absent here.
  { id: "reconcileBackgroundResponses" },
];
