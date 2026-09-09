class Registry {
  static get current() {
    return import("./aliased-static-getter-loaded.mts");
  }
}

const Alias = Registry;

export function run() {
  return Alias.current;
}

run();
