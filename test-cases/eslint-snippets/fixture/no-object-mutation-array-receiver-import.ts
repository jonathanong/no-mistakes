import type { Array } from "custom-types";
export function custom(this: Array<number>) { this.push(1); }
