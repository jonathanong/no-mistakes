import { RuntimeError } from './errors';
declare function opaque(): typeof RuntimeError;
const RuntimeAlias: typeof RuntimeError = opaque();
// Runtime annotations describe assignability, not the initializer's identity.
const RuntimeFacade: { Item: typeof RuntimeError } = opaqueObject();
declare function opaqueObject(): { Item: typeof RuntimeError };
export class RuntimeAliasChild extends RuntimeAlias {}
export class RuntimeMemberChild extends RuntimeFacade.Item {}
