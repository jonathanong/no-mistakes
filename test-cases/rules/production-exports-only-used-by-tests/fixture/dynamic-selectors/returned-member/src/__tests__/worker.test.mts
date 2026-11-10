function importWorker() {
  vi.resetModules();
  return import("../worker.mjs");
}
const loaded = await importWorker();
loaded.default();
