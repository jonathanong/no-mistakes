const [{ work }] = await getPromise().all([import("./worker.mjs")]);
