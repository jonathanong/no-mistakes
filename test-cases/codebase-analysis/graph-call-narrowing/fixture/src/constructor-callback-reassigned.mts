async function target() {
  await import("./constructor-callback-loaded.mts");
}

class Service {
  constructor(_cb: () => void) {}
}

let cb = target;
cb = async () => {};
new Service(cb);
