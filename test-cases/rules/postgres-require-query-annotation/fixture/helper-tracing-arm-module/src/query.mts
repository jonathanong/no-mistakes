import { mutate, send } from './state.mjs';
// Returned callbacks must resolve the same module state initialized in this arm.
const callback = flag ? (mutate(), () => send()) : (() => {});
const consumed = opaque(callback);
