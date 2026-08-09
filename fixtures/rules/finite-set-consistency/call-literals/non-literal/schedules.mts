const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

const runtimeGenerationId = "reconcileRuntimeGenerations";
ai_agents.upsertJobScheduler("reconcileAutoDispatchJudgements");
ai_agents.upsertJobScheduler(runtimeGenerationId);
ai_agents.upsertJobScheduler(`reconcileBackgroundResponses`);
