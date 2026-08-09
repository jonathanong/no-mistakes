declare const ai_agents:
  | { upsertJobScheduler(id: string): void }
  | undefined;

// Optional chaining does not guarantee that the configured target is invoked.
ai_agents?.upsertJobScheduler("ignored-member-chain");
ai_agents.upsertJobScheduler?.("ignored-optional-call");
