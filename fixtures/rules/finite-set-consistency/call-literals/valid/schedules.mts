const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

ai_agents.upsertJobScheduler("reconcileAutoDispatchJudgements");
ai_agents.upsertJobScheduler(`reconcileRuntimeGenerations`);
ai_agents.upsertJobScheduler('reconcileBackgroundResponses');

// A different callee must not contribute to the scheduler set.
other_agents.upsertJobScheduler("not-an-ai-agent-scheduler");
// Multi-hop members are outside the configured one-level callee syntax.
namespace.ai_agents.upsertJobScheduler("not-a-one-level-call");
