const [{ [key]: work }] = await Promise.all([import("./worker.mjs")]);
