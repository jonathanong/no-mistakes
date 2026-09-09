function target() {}

const api = { run: target };
const facade = api;

export function callThroughFacade() {
  facade.run();
}
