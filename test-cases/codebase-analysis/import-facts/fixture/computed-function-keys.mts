const loaders = {
  [require("./key.mts")]: function () {
    import("./loaded.mts");
  },
};

class Loader {
  [require("./method-key.mts")]() {
    import("./loaded.mts");
  }
}

new Loader();
