class Registry {
  static get current() {
    return import("./computed-static-getter-loaded.mts");
  }
}

const name = "current";

export function run() {
  return Registry[name];
}

run();
