import { original } from "./api";
// v0.95.0 sees the first callee as Identifier, but not the second instantiation expression.
const direct = value => original<string>(value);
const parenthesized = value => (original<string>)(value);
