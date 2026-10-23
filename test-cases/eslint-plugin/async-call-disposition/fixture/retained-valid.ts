import { enqueueEmail } from "@app/jobs";
import { vi } from "vitest";
import { jest } from "@jest/globals";
import * as vitest from "vitest";

vi.spyOn(service, "send").mockImplementation((id) => enqueueEmail(id));
vi.spyOn(service, "send").mockImplementationOnce(function (id) { return enqueueEmail(id); });
jest.fn((id) => enqueueEmail(id));
vitest.vi.fn((id) => { const job = enqueueEmail(id); return job; });
const mock = vi.fn();
mock.mockImplementation((id) => { const job = enqueueEmail(id); return job; });

export async function issueSample() {
  const pending: Promise<unknown>[] = [];
  vi.spyOn(service, "send").mockImplementation((id) => {
    const job = enqueueEmail(id);
    pending.push(job);
    return job;
  });
  try { return await run(); } finally { await Promise.all(pending); }
}

export async function collected() {
  const pending = [];
  const job = enqueueEmail("1");
  pending.push(job);
  await Promise.all(pending);
}

export async function settled() {
  const pending = [];
  pending.push(enqueueEmail("1"));
  await Promise.allSettled(pending);
}
vi.fn((() => { const job = enqueueEmail("1"); return job; }) as Impl);
const api = vi;
api.fn((id) => enqueueEmail(id));
const nested = vi.fn().mockImplementation(() => {});
nested.mockImplementationOnce((id) => enqueueEmail(id));
export async function aliasesAndWrappers() {
  const pending = [];
  const first = enqueueEmail("1");
  const second = first;
  pending.push(second);
  pending.push(enqueueEmail("2"));
  return Promise.allSettled(pending);
}
import * as jestGlobals from "@jest/globals";
jestGlobals.jest.fn((id) => enqueueEmail(id));
