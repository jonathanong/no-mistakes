// Unrelated uncertainty must not erase an established test-only export finding.
const moduleName = "./unrelated.mjs";
void import(moduleName);
