const api = {
  load() {
    return import("./object-setter-data-loaded.mts");
  },
};

api.load = async () => {};
api.load();
