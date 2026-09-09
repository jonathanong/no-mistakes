const registry = {
  get current() {
    return import("./object-getter-loaded.mts");
  },
};

export function run() {
  return registry.current;
}

run();
