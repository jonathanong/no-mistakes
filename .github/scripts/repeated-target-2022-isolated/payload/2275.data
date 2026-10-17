import { api, block, computedScope, updatedScope, matchingScope } from '@vendor/valkey';
function replacement() {}
let unused;
function scopeApi(client) { client.get(); }
function scopeBlock(client) { client.get(); }
function scopeComputed(client) { client.get(); }
function scopeUpdated(client) { client.get(); }
function scopeMatching(client) { client.get(); }
// A class's private name must not become an earlier write to the import.
({ a: unused = class api { static { api.get = replacement; } }, b: unused = scopeApi(api), c: api.get } = {});
// Static-block declarations have their own lexical binding, too.
({ a: unused = class Local { static { const block = {}; block.get = replacement; } }, b: unused = scopeBlock(block), c: block.get } = {});
// Computed keys and updates use the same scope-qualified write receipts.
({ [class LocalComputed { static { const computedScope = {}; computedScope.get = replacement; } }]: unused, b: unused = scopeComputed(computedScope), c: computedScope.get } = {});
({ a: unused = class LocalUpdated { static { const updatedScope = { get: 0 }; updatedScope.get++; } }, b: unused = scopeUpdated(updatedScope), c: updatedScope.get } = {});
// An unshadowed static-block write really does invalidate the later read.
({ a: unused = class Real { static { matchingScope.get = replacement; } }, b: unused = scopeMatching(matchingScope), c: matchingScope.get } = {});
