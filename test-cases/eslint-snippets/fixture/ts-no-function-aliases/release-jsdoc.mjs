import { original } from "./api";
/** @param {Code} value @returns {value is Code} */
export function narrowed(value) { return original(value); }
/** @type {(value: Code) => boolean} */
export const contextual = value => original(value);
const inline = /** @param {Code} value */ value => original(value);
/** @template T @param {T} value @returns {T} */
const generic = value => original(value);
/** @overload @param {Code} value @returns {boolean} */
export function overloaded(value) { return original(value); }
// An ordinary explanation does not add a type contract.
export const plain = value => original(value);
/** A description alone also leaves the function's contract unchanged. */
export const described = value => original(value);
