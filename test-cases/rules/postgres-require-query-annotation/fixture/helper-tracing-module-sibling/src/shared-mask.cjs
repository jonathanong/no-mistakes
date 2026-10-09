// A lazy module may share this argument object across sibling alternatives.
function capture(callback) { return arguments; }
const shared = capture(() => '/* annotation */ SELECT 1');
