const loaders = {
  load() {
    import("./loaded.mts");
  },
  fallback: function () {
    import("./loaded.mts");
  },
};

loaders.load();
