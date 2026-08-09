const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

const runtimeGenerationId = "reconcileRuntimeGenerations";
ai_agents.upsertJobScheduler("reconcileAutoDispatchJudgements");
// no-mistakes-disable-next-line finite-set-consistency
ai_agents.upsertJobScheduler(runtimeGenerationId);
ai_agents.upsertJobScheduler(`reconcileBackgroundResponses`);
