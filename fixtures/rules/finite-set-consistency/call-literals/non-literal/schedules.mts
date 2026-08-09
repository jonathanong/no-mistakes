const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

const runtimeGenerationId = "reconcileRuntimeGenerations";
ai_agents.upsertJobScheduler("reconcileAutoDispatchJudgements");
// The variable argument intentionally verifies fail-closed static-ID extraction.
ai_agents.upsertJobScheduler(runtimeGenerationId);
ai_agents.upsertJobScheduler(`reconcileBackgroundResponses`);
