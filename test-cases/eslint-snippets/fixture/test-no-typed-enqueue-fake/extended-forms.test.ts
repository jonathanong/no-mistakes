import { vi } from "vitest";
import type { Mock, MockedFunction } from "vitest";
import type { Mock as CustomMock } from "./custom-types";
import * as vitest from "vitest";
import * as queues from "./queue";
import { enqueueEmail } from "./queue";
import { enqueueAudit } from "./audit";

vi.fn<typeof queues.enqueueEmail>();
jest.fn<typeof queues.enqueueEmail>();
vi.fn<Parameters<typeof enqueueEmail>, ReturnType<typeof enqueueEmail>>();
jest.fn<ReturnType<typeof queues.enqueueEmail>, Parameters<typeof queues.enqueueEmail>>();
vi.fn<(...args: Parameters<typeof enqueueEmail>) => Promise<void>>();
let fakeA: Mock<typeof enqueueEmail>;
let fakeB: MockedFunction<typeof queues.enqueueEmail>;
let fakeC: vitest.Mock<Parameters<typeof enqueueEmail>, Promise<void>>;
let fakeD: jest.MockedFunction<typeof enqueueEmail>;
const castA = vi.mocked(enqueueEmail);
const castB = jest.mocked(enqueueEmail);
const castC = vi.mocked(queues.enqueueEmail);

let custom: CustomMock<typeof enqueueEmail>;
let otherQualified: queues.Other<typeof enqueueEmail>;
vi.fn<(recipient: string) => ReturnType<typeof enqueueEmail>>();
vi.fn<typeof queues>();
vi.mocked(queues);
vi.mocked(() => {});
vi.mocked(enqueueAudit);
function shadowed(queues: { enqueueEmail: () => void }) {
  type Mock<T> = T;
  type Parameters<T> = T;
  let local: Mock<typeof enqueueEmail>;
  vi.fn<typeof queues.enqueueEmail>();
  vi.fn<Parameters<typeof enqueueEmail>, void>();
  vi.mocked(queues.enqueueEmail);
}
