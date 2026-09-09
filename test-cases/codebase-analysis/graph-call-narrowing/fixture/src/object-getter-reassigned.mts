let registry = {
  get current() {
    return import("./object-getter-reassigned-loaded.mts");
  },
};
registry = {};

export function run() {
  return registry.current;
}

run();
