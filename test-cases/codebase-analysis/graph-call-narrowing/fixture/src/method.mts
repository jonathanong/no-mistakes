class Loader {
  load() {
    import("./uncalled.mts");
  }
}

new Loader();
