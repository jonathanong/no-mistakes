async function run(Promise) { const [{ work }] = await Promise.all([import("./worker.mjs")]); }
