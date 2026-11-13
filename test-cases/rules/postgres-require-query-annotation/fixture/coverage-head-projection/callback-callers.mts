import sql from "sql-template-strings";
import {read} from "@bench/db";
import {page} from "./callbacks.mjs";
function literalAppend(q) { q.append(sql` WHERE id = 1`); }
page(sql`/* literalAppend */ SELECT 1`, q => { literalAppend(q, 1); return read(q); }); // low: true
function parameterAppend(q, id) { q.append(sql` WHERE id = ${id}`); }
page(sql`/* parameterAppend */ SELECT 1`, q => { parameterAppend(q, 1); return read(q); }); // low: true
export function exportedAppend(q, id) { q.append(sql` WHERE id = ${id}`); }
page(sql`/* exportedAppend */ SELECT 1`, q => { exportedAppend(q, 1); return read(q); }); // low: true
async function asyncAppend(q) { q.append(sql` WHERE id = 1`); }
page(sql`/* asyncAppend */ SELECT 1`, q => { asyncAppend(q, 1); return read(q); }); // low: false
function* generatorAppend(q) { q.append(sql` WHERE id = 1`); }
page(sql`/* generatorAppend */ SELECT 1`, q => { generatorAppend(q, 1); return read(q); }); // low: false
function defaultAppend(q, id = 1) { q.append(sql` WHERE id = ${id}`); }
page(sql`/* defaultAppend */ SELECT 1`, q => { defaultAppend(q, 1); return read(q); }); // low: false
function restAppend(q, ...ids) { q.append(sql` WHERE id = 1`); }
page(sql`/* restAppend */ SELECT 1`, q => { restAppend(q, 1); return read(q); }); // low: false
function destructuredAppend({q}) { q.append(sql` WHERE id = 1`); }
page(sql`/* destructuredAppend */ SELECT 1`, q => { destructuredAppend(q, 1); return read(q); }); // low: false
function doubleUse(q) { q.append(sql` WHERE id = ${q}`); }
page(sql`/* doubleUse */ SELECT 1`, q => { doubleUse(q, 1); return read(q); }); // low: false
function opaqueInterpolation(q) { q.append(sql` WHERE id = ${runtimeValue}`); }
page(sql`/* opaqueInterpolation */ SELECT 1`, q => { opaqueInterpolation(q, 1); return read(q); }); // low: false
function callInterpolation(q) { q.append(sql` WHERE id = ${compute()}`); }
page(sql`/* callInterpolation */ SELECT 1`, q => { callInterpolation(q, 1); return read(q); }); // low: false
function plainSuffix(q) { q.append(" WHERE id = 1"); }
page(sql`/* plainSuffix */ SELECT 1`, q => { plainSuffix(q, 1); return read(q); }); // low: false
function optionalAppend(q) { q.append?.(sql` WHERE id = 1`); }
page(sql`/* optionalAppend */ SELECT 1`, q => { optionalAppend(q, 1); return read(q); }); // low: false
function wrongReceiver(q) { other.append(sql` WHERE id = 1`); }
page(sql`/* wrongReceiver */ SELECT 1`, q => { wrongReceiver(q, 1); return read(q); }); // low: false
function returnAppend(q) { return q.append(sql` WHERE id = 1`); }
page(sql`/* returnAppend */ SELECT 1`, q => { returnAppend(q, 1); return read(q); }); // low: false
function untrustedAppend(q) { q.append(unknownTag` WHERE id = 1`); }
page(sql`/* untrustedAppend */ SELECT 1`, q => { untrustedAppend(q, 1); return read(q); }); // low: false
