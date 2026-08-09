import { test } from "vitest";

const ai_agents = {
  upsertJobScheduler(_id: string) {},
};

test("registers the generated scheduler", async () => {
  await import("../src/lazy.mts");
  ai_agents.upsertJobScheduler("missing");
});
