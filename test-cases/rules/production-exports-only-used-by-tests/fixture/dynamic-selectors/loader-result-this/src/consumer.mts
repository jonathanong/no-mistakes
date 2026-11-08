const registry = { x: () => import("./worker.mjs") };
const m = await registry.x(); m.work();
