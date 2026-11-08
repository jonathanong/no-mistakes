const registry = { x: () => import("./worker.mjs"), y: () => import("./other.mjs") };
await registry["y"]();
