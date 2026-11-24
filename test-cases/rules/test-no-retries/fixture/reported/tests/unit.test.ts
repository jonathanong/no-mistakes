jest.retryTimes(3);

test("retries inline", { retry: 2 }, () => {});

describe("suite retries", { retry: 1 }, () => {});

// no-mistakes-disable-next-line test-no-retries
jest.retryTimes(2);
