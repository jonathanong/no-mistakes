export function execute() { return 1; }
// A top-level occurrence exercises module ownership without satisfying producers.
const unrelatedModuleValue = Number(1);
