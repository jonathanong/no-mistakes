import { primary } from '@vendor/valkey';
export function factory() { return ({ client = primary } = {}) => client.get(); }
// This module write precedes every importing module's invocation.
primary.get = function replacement() {};
