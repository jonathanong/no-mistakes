import * as React from 'react';
import { ExternalError, OpaqueFactory, MissingBase } from 'external-errors';
// A package declaration proves Component is a non-Error class, so this stays quiet.
export class Component extends React.Component {}
export class ImportedError extends ExternalError {}
export class Missing extends MissingBase {}
// A resolved function is not proof of non-Error class ancestry.
export class Opaque extends OpaqueFactory {}
// no-mistakes-disable-next-line unconstructed-error-class -- This opaque external base is intentional.
export class SuppressedMissing extends MissingBase {}
export function render() { return React.createElement(Component); }
import CjsError from 'cjs-error';
export class ImportedCjsError extends CjsError {}
