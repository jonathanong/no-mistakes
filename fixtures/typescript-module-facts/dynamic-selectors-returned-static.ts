const load = () =>
  class Recovered {
    static {
      return import("./static-recovered");
    }
  };
void load;
