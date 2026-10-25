// Cyclic initializers never prove framework identity; analysis must terminate.
const first = second;
const second = first;
first.mock("./a");
