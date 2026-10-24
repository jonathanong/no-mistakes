import { HttpError } from 'external-errors';
export class ExternalLeaf extends HttpError {}
export class ExternalBase extends HttpError {}
// Unknown ancestry propagates through a repository inheritance edge.
export class ExternalChild extends ExternalBase {}
export class KnownExternal extends HttpError {}
new KnownExternal();
