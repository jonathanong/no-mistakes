const { jest: timers } = require("@jest/globals");

expect(Date.now() - start).toBeLessThan(200);
timers.useFakeTimers();
