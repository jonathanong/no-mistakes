const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

ai_agents.upsertJobScheduler("reconcileAutoDispatchJudgements");
ai_agents.upsertJobScheduler(`reconcileRuntimeGenerations`);
ai_agents.upsertJobScheduler('reconcileBackgroundResponses');

// A different callee must not contribute to the scheduler set.
other_agents.upsertJobScheduler("not-an-ai-agent-scheduler");
