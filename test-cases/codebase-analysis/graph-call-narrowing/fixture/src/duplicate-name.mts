async function load() {
  await import("./called.mts");
}

function wrapper() {
  async function load() {
    await import("./uncalled.mts");
  }

  return load;
}

load();
