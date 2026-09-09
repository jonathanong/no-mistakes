class Registry {
  static get value() {
    return import("./paired-static-accessor-getter-loaded.mts");
  }

  static set value(_next: unknown) {
    import("./paired-static-accessor-setter-loaded.mts");
  }
}

Registry.value;
Registry.value = 1;
