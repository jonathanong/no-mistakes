class Registry {
  static get current() {
    return import("./computed-static-getter-loaded.mts");
  }
}

export function run() {
  return Registry["current"];
}

run();
