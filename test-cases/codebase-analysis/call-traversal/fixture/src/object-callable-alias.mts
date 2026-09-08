function objectTarget() {}

const api = { run: objectTarget };

export function callThroughObject() {
  api.run();
}
