const source = {
  run() {
    return import("./object-spread-loaded.mts");
  },
};

const api = {
  ...source,
  run() {},
};

api.run();
