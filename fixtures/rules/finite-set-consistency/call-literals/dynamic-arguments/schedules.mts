const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

const schedulerId = "reconcileRuntimeGenerations";
const schedulerIds = [schedulerId];
ai_agents.upsertJobScheduler();
ai_agents.upsertJobScheduler(...schedulerIds);
ai_agents.upsertJobScheduler(`reconcile-${schedulerId}`);
