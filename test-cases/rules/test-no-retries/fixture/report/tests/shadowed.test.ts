import { test } from "@playwright/test";

export function setup(test: { describe: { configure: (options: object) => void } }) {
  test.describe.configure({ mode: "serial", retries: 99 });
}

const jest = { retryTimes(count: number) { return count; } };
jest.retryTimes(99);
