async function boot() {
  runner[method]();
}

async function hidden() {
  await import("./uncalled.mts");
}

boot();
