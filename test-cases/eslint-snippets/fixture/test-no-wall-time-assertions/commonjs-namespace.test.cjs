const framework = require("@jest/globals");

expect(Date.now() - start).toBeLessThan(200);
framework.jest.useFakeTimers();
