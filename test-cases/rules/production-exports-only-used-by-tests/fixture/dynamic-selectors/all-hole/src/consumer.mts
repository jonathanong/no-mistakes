const [, { work }] = await Promise.all([0, import("./worker.mjs")]); work();
