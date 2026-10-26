declare const ai_agents: {
  upsertJobScheduler(id: string, options: object): void;
};

declare function getRuntimeId(): string;

ai_agents.upsertJobScheduler(getRuntimeId(), {});
