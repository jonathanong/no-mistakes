class Registry {
  static get value() {
    return import("./paired-static-accessor-getter-loaded.mts");
  }

  static set value(_next: unknown) {
    import("./paired-static-accessor-setter-loaded.mts");
  }
}

export function run() {
  return Registry.value;
}

run();
