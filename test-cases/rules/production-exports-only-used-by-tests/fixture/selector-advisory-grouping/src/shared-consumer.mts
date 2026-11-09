import * as values from './shared.mjs';
function invoke(t: typeof values, name: keyof typeof values) { return t[name](); }
invoke(values, 'first');
