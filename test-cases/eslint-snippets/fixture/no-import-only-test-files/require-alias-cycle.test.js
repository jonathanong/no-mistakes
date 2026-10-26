// Never pass this fixture to Oxlint: its TDZ-cycle analysis aborts even without this plugin.
// Cyclic initializers never prove require identity; analysis must terminate.
const load = again;
const again = load;
load("./a.test");
