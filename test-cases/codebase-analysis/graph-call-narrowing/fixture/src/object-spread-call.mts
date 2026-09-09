const source = {
  run() {
    return import("./object-spread-loaded.mts");
  },
};

({ ...source }).run();
