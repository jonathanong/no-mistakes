const [{ work }] = await Promise.all([...values, import("./worker.mjs")]);
