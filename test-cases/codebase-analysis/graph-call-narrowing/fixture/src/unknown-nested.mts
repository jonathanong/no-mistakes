async function boot() {
  factory().invoke();
}

async function hidden() {
  await import("./uncalled.mts");
}

boot();
