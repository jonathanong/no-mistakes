import { HttpError } from 'external-errors';
// no-mistakes-disable-next-line unconstructed-error-class: external ownership
export class SuppressedExternal extends HttpError {}
export function line(ctor: any) { return new ctor(); } // no-mistakes-disable-line unconstructed-error-class: dynamic registration
// no-mistakes-disable-next-line unconstructed-error-class: explicit plugin boundary
export function next(ctor: any) { return new ctor(); }
export namespace Plugins {
  // no-mistakes-disable-next-line unconstructed-error-class: loaded by the plugin host
  export class SuppressedNamespace extends Error {}
}
export const registered = Plugins;

// One line can exempt both the class uncertainty and its unresolved constructor.
export class SuppressedFactory extends Error { static make(ctor: any) { return new ctor(); } } // no-mistakes-disable-line unconstructed-error-class: plugin factory
