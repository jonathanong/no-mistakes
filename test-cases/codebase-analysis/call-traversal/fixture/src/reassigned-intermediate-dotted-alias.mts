function target() {}
function other() {}

let invoke = target;
const calls = { run: invoke };
invoke = other;

// Intermediate reassignment invalidates the hop; stay conservative.
export function callAfterInvokeReassign() {
  calls.run();
}
