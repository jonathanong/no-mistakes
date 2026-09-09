const run = () => {
  helper();

  async function helper() {
    await import("./called.mts");
  }
};

run();
