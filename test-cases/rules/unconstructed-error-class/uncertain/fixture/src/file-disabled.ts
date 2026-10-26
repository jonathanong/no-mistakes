// no-mistakes-disable-file unconstructed-error-class: external plugin module
import { HttpError } from 'external-errors';
export class FileDisabled extends HttpError {}
export function build(ctor: any) { return new ctor(); }
