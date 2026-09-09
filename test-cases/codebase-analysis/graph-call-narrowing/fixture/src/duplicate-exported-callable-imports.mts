{
  async function target() {
    await import("./uncalled.mts");
  }
}

async function target() {
  await import("./called.mts");
}

export { target };
target();
