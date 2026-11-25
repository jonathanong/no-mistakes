import { jest } from "@jest/globals";

jest.retryTimes(-1);
jest.retryTimes(0x02);
jest.retryTimes(inCI ? maxAttempts : 0);
jest.retryTimes(inCI ? 0 : maxAttempts);
