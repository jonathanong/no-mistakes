async function target() {
  await import("./called.mts");
}

async function shadowedTarget() {
  await import("./uncalled.mts");
}

function run() {
  {
    // This binding is not visible outside its lexical block.
    const invoke = target;
    target = shadowedTarget;
    invoke();
  }
  const invoke = shadowedTarget;
  {
    const invoke = () => {};
    invoke();
  }
}

run();
