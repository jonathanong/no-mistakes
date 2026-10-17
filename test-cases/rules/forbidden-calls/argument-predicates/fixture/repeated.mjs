// A configured callee can mutate a captured carrier before its next use.
function execute(value) { value.length = 0; }
const captured = ['--root'];
export function repeat() { execute(captured); }
const looped = ['--root'];
for (let i = 0; i < 2; i++) { execute(looped); }
export function fresh() { const local = ['--root']; execute(local); }
class RepeatedInstance { value = execute(captured); }
