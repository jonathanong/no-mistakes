import sql from 'sql-template-strings';
import { sql as customQuery } from '@consumer/db';
import { sql as subQuery } from '@consumer/db/sql';
import { sql as untrusted } from '@other/db';
import * as namespace from '@consumer/db';
import type { sql as typeOnly } from '@consumer/db';
import { type sql as inlineType } from '@consumer/db';
import { outerWrite, arrowWrite, loopWrite, updateWrite, destructureWrite, restWrite, defaultWrite, defaultParamWrite, arrowParamWrite } from './providers.mjs';

// Local declarations must never revoke another scope's imported tag.
function shadows(customQuery: unknown, { subQuery }: {subQuery: unknown}, ...rest: unknown[]) {
  customQuery = rest;
  subQuery++;
  let sql = otherTag;
  sql = otherTag;
  const [untrusted, ...remaining] = rest;
  function helper() { return untrusted; }
}
const arrow = ({customQuery}, [subQuery, ...rest]) => { customQuery = rest; subQuery++; };
function hoisted() { outerWrite = otherTag; var outerWrite; }
function expressionSelf() { const inner = function outerWrite() { outerWrite = otherTag; }; }
{ let customQuery = otherTag; customQuery = otherTag; }
for (let customQuery = otherTag; customQuery; customQuery++) { customQuery = otherTag; }
for (const subQuery of providers) { subQuery = otherTag; }
for (let sql in providers) { sql = otherTag; }
try { throw providers; } catch ({customQuery, ...rest}) { customQuery = rest; }
switch (providers) { case 1: customQuery = otherTag; break; default: let customQuery = otherTag; }
function classScope() { class customQuery { method() { customQuery = otherTag; } static { var sql; sql = otherTag; } } }
const holder = { method() { let customQuery; customQuery = otherTag; } };

// Writes without a shadow still mutate the module binding, even in a closure.
function mutate() { outerWrite = otherTag; }
const mutateArrow = () => { arrowWrite = otherTag; };
for (loopWrite of providers) {}
updateWrite++;
({ value: destructureWrite, restWrite, ...restWrite } = providers);
[defaultWrite = fallback] = providers;
moduleObject.value = providers;
moduleObject[property] = providers;
class PrivateWrites { #value; method() { this.#value = providers; } }
function helper() { helper = otherHelper; }
var helper = otherHelper;
export function exportedHelper() { return sql`/* named */ SELECT 1`; }
for (var exportedHelper of providers) {}
globalWrite = providers;
enum ScopeEnum { Value }
const { x: scoped = fallback, ...restScope } = providers;

// Namespace locals obey the same shadowing rules as function locals.
namespace Tags { let customQuery; customQuery = otherTag; var sql; sql = otherTag; }
namespace Writes { outerWrite = otherTag; }

namespace Imports { import customQuery = Other.tag; customQuery = otherTag; }
function namespaces() { namespace customQuery { export const value = 1; } customQuery = otherTag; }

// Body var declarations are absent from default parameter evaluation.
function defaultParameters(value = (defaultParamWrite = otherTag)) { var defaultParamWrite; }
const defaultArrow = (value = (arrowParamWrite = otherTag)) => { var arrowParamWrite; };
