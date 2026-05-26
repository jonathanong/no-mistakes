const loader = function namedLoader() {
  return import("./loaded.mts");
};

loader();
