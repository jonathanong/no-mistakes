function configure(ai_agents: { upsertJobScheduler(id: string): void }) {
  ai_agents.upsertJobScheduler("reconcileRuntimeGenerations");
}

configure({ upsertJobScheduler() {} });
