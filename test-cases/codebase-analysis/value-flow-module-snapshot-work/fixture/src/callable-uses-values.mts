function stored() {}
function direct() {}
globalThis.hook = stored;
const container = { stored };
function returned() { return stored; }
external(stored);
direct(external(stored));
// A shadowed non-callable parameter must not mark another lexical binding.
function shadow(stored) { return stored; }
(direct as (() => void))();
