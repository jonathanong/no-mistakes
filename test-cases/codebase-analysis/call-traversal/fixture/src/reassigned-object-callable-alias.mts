function replacement() {}

const api = { run() {} };

function replace() {
  api.run = replacement;
}

export function callAfterReplacement() {
  api.run();
}
