export async function direct(deps: any) { await consume(deps); }
export async function selected(deps: any) { await callback(deps.job); }
export async function renamed(deps: any) { await consumeRenamed({ renamed: deps.job }); }
export async function invalidObject(deps: any) { await consume({ job: deps.job, ...other }); }
export async function duplicate(deps: any) { await consume({ job: deps.job, job: replacement }); }
export async function dynamicObject(deps: any) { await consume({ [key]: deps.job }); }
export async function methodObject(deps: any) { await consume({ job: deps.job, method() {} }); }
export async function getterObject(deps: any) { await consume({ job: deps.job, get other() { return 1; } }); }
export async function spreadArray(deps: any) { await consume(...deps); }
export async function unawaited(deps: any) { consume(deps); }
export async function rhs(deps: any) { await consume({ job: replacement ?? deps.job }); }
export async function nested(deps: any) { await run(async () => await deps.job()); }
export async function overwritten(deps: any) { await consume({ job: replacement, ...deps }); }
export async function otherSpread(deps: any) { await consume({ ...deps, ...other }); }
async function consume(deps: any) { await deps.job(); }
async function consumeRenamed(deps: any) { await deps.renamed(); }
async function callback(callback: any) { await callback(); }
async function run(callback: any) { await callback(); }
declare const other: any, replacement: any, key: string;
export async function spreadComputed(deps: any) { await consume({ ...deps, [key]: 1 }); }
export async function spreadMethod(deps: any) { await consume({ ...deps, method() {} }); }
export async function spreadGetter(deps: any) { await consume({ ...deps, get other() { return 1; } }); }
export { callback as "stringConsumer" };
export { absent } from "./cross-file-not-found";
