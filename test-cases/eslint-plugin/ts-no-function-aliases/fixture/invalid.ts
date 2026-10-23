import original, { work } from "./api";
import * as api from "./api";
export const noArgs = () => original();
export const withArg = value => original(value);
export const rest = (...args) => original(...args);
export const member = value => api.create(value);
export const call = value => work.call(undefined, value);
export const apply = value => work.apply(undefined, [value]);
export function exported(value: string): string { return value; }
export const typed = (value: string): string => exported(value);
