const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

const runtimeGenerationId = "reconcileRuntimeGenerations";
ai_agents.upsertJobScheduler("reconcileAutoDispatchJudgements");
// The dynamic call is intentionally suppressed, while the retained static
// call value must still participate in the finite-set comparison.
// no-mistakes-disable-next-line finite-set-consistency
ai_agents.upsertJobScheduler(runtimeGenerationId);
