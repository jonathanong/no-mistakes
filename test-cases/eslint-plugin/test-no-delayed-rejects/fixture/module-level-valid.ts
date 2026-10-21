import { expect } from "vitest";

declare function releaseLock(): Promise<void>;
declare function updateCommunityAgentPrompt(input: unknown): Promise<unknown>;
declare const input: unknown;

const update = updateCommunityAgentPrompt(input);
const rejection = update.catch((error: unknown) => error);
await releaseLock();
await expect(rejection).resolves.toMatchObject({ status: 403 });
