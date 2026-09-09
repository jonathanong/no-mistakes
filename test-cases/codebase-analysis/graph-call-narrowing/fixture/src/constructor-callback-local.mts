async function target() {
  await import("./constructor-callback-loaded.mts");
}

class Service {
  constructor(_cb: () => void) {}
}

new Service(target);
