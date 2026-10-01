// Does not parse. A test file's alias of a namespace keeps its classes quiet,
// so the rule cannot report `Errors.Dead` without reading this file and must
// stop instead.
import { Errors } from "../errors";

export const all = Errors;

const unterminated = {;
