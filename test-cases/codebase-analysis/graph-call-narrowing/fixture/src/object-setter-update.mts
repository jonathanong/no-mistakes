const api = {
  set current(_next: unknown) {
    import("./object-setter-loaded.mts");
  },
};

api.current++;
--api.current;
