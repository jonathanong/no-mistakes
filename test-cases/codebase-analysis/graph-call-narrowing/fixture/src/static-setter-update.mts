class Registry {
  static set current(_next: unknown) {
    import("./static-setter-loaded.mts");
  }
}

// Prefix/postfix updates invoke the setter; keep its runtime import.
Registry.current++;
--Registry.current;
