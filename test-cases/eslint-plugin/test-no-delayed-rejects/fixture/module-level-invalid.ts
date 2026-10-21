import { expect } from "vitest";

declare function releaseLock(): Promise<void>;
declare function updateCommunityAgentPrompt(input: unknown): Promise<unknown>;
declare const input: unknown;

const update = updateCommunityAgentPrompt(input);
await releaseLock();
await expect(update).rejects.toMatchObject({ status: 403 });
