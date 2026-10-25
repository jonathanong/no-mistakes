// Cyclic initializers never prove framework identity; analysis must terminate.
const first = second;
const second = first;
first.mock("./a");

// Multiple local definitions do not prove framework identity.
var api;
var api;
api.mock("./redeclared");
