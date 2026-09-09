const api = {
  set current(_next: unknown) {
    import("./object-setter-loaded.mts");
  },
};

api.current = 1;
api["current"] = 2;
