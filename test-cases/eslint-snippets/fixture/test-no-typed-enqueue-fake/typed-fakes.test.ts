import { vi as mock } from "vitest";
import { jest as jestMock } from "@jest/globals";
import * as vitest from "vitest";
import { vi as unrelatedMock } from "./helpers";
import dispatch, { enqueueEmail as sendMail, enqueueEmail, formatEmail as enqueuePreview } from "./queue";
import type { enqueueNotice } from "./queue";
import { enqueueAudit } from "./audit";

vi.fn<typeof enqueueEmail>();
jest.fn<typeof enqueueEmail>();
mock.fn<typeof sendMail>();
jestMock.fn<typeof sendMail>();
vitest.vi.fn<typeof sendMail>();
mock.fn<typeof enqueueNotice>();
mock.fn<typeof enqueueAudit>();
mock.fn<typeof enqueuePreview>();
mock.fn<(...args: [address: string, options: { jobId: string }]) => Promise<void>>();
mock.spyOn({ sendMail }, "sendMail");
unrelatedMock.fn<typeof sendMail>();
vitest.fn<typeof sendMail>();

function shadowTarget(sendMail: () => void) {
  mock.fn<typeof sendMail>();
}

function shadowRunner(mock: typeof vi, jest: typeof jestMock) {
  mock.fn<typeof sendMail>();
  jest.fn<typeof sendMail>();
}

function localTarget() {
  const enqueueEmail = () => {};
  mock.fn<typeof enqueueEmail>();
}

function separateTypeNamespace() {
  type sendMail = string;
  mock.fn<typeof sendMail>();
}

// no-mistakes-disable-next-line test-no-typed-enqueue-fake -- Verifies the directive.
mock.fn<typeof sendMail>();
mock.fn<typeof sendMail>(); // no-mistakes-disable-line test-no-typed-enqueue-fake -- Verifies the directive.

mock.fn<typeof dispatch>();
mock.fn<typeof setTimeout>();
(() => mock)().fn<typeof sendMail>();
