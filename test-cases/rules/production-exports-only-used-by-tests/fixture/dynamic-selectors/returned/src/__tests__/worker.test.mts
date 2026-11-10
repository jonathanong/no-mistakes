function importWorker() {
  vi.resetModules();
  return import("../worker.mjs");
}
const { default: main } = await importWorker();
main();
