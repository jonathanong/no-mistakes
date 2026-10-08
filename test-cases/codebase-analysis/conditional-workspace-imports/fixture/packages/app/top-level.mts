await Promise.all([
  import("./relative.mts"),
  import("@fx/lib/target-a"),
  import("@fx/lib/target-b"),
  import("#local/target"),
]);
export {};
