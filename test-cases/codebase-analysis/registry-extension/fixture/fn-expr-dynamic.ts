import { registry } from "./registry";

// Function-expression registrants returning a dynamic import.
registry.register(function () {
  return import("./plugins/fn-a");
});
registry.register(function () {
  return import("./plugins/fn-b");
});
