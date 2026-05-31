export function outer() {
  return class {
    run() {
      import("./loaded.mts");
    }
  };
}
