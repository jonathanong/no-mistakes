import { jest as legacyJest } from "@jest/globals";

legacyJest.retryTimes(3);
legacyJest.retryTimes(0);
legacyJest.retryTimes(retriesFromEnvironment);
jest.retryTimes(2);
// no-mistakes-disable-next-line test-no-retries
legacyJest.retryTimes(9);
