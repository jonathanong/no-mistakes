const spec = "@example/jobs";
const V = require(spec);
V.vi.fn<typeof V.enqueue>();
