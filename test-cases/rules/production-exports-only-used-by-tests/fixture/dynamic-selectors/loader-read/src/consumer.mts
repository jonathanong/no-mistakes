const registry = { x: () => import("./worker.mjs") };
const selected = (await registry.x()).default;
