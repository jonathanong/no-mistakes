// Cyclic initializers never prove require identity; analysis must terminate.
const load = again;
const again = load;
load("./a.test");
