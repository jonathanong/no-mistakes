const [, { work }] = await Promise.all([import("./worker.mjs"), import("./other.mjs")]); work();
