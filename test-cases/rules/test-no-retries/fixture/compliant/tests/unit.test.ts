jest.retryTimes(0);
test("no retries", { retry: 0 }, () => {});
test.describe.configure({ retries: 0 });
