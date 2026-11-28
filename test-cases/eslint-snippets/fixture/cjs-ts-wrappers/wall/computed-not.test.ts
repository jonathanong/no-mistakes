const spec = "node:perf_hooks";
const clock = require(spec);
expect(clock.now() - 0)["not"].toBeLessThan(200);
