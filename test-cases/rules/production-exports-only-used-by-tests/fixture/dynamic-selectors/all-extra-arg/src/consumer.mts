const [{ work }] = await Promise.all([import("./worker.mjs")], extra);
