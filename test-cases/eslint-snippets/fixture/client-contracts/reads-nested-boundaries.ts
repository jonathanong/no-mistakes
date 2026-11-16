import { cache as kv } from '@app/cache';

async function dependent(k1: string) {
  const a = JSON.parse((await kv.get(k1)) ?? '{}');
  const b = Boolean(await kv.get(a.nextKey));
}

async function ambiguous(k1: string, k2: string) {
  const a = Number(await kv.ttl(k1));
  const b = [await kv.get(k1), await kv.get(k2)];
  const c = Number(await kv.ttl(k2));
}

async function nestedReads(k1: string, k2: string) {
  const a = Number(await kv.ttl(k1));
  const b = await kv.get(await kv.get(k2));
  const c = Number(await kv.ttl(k2));
}

async function callbacks(keys: string[]) {
  const a = Number(await kv.ttl('first'));
  const b = keys.map(async (key) => Boolean(await kv.exists(key)));
  const c = Number(await kv.ttl('last'));
}

async function nestedFunction(k1: string) {
  const a = Number(await kv.ttl(k1));
  const b = async function () { return await kv.get('inside'); };
  const c = Number(await kv.ttl(k1));
}

async function logicalRight(k1: string, value: unknown) {
  const a = Number(await kv.ttl(k1));
  const b = value ?? (await kv.get('fallback'));
  const c = Number(await kv.ttl(k1));
  const d = value || (await kv.get('alternative'));
  const e = Number(await kv.ttl(k1));
  const f = value && (await kv.get('conditional'));
}

async function conditionalBranches(k1: string, value: unknown) {
  const a = Number(await kv.ttl(k1));
  const b = value ? await kv.get('yes') : 'no';
  const c = Number(await kv.ttl(k1));
  const d = value ? 'yes' : await kv.get('no');
}

async function conditionalExtra(k1: string, flag: unknown) {
  const a = Number(await kv.ttl(k1));
  const b = [await kv.get('always'), flag && await kv.get('conditional')];
  const c = Number(await kv.ttl(k1));
}

async function optionalArgument(k1: string, consume?: (value: unknown) => unknown) {
  const a = Number(await kv.ttl(k1));
  const b = consume?.(await kv.get('conditional'));
  const c = Number(await kv.ttl(k1));
}

async function optionalComputed(k1: string, value: unknown) {
  const a = Number(await kv.ttl(k1));
  const b = value?.[await kv.get('conditional')];
  const c = Number(await kv.ttl(k1));
}

async function optionalRead(k1: string, k2: string) {
  const a = Number(await kv.ttl(k1));
  const b = Boolean(await kv?.get(k2));
  const c = Number(await kv.ttl(k1));
  const d = Boolean(await kv.get?.(k2));
}

async function logicalAssignment(k1: string, value: unknown) {
  const a = Number(await kv.ttl(k1));
  const b = Boolean(value ||= await kv.get('fallback'));
  const c = Number(await kv.ttl(k1));
}

async function assignmentDefault(k1: string, source: unknown) {
  const a = Number(await kv.ttl(k1));
  let value;
  const b = Boolean(({ value = await kv.get('fallback') } = source));
  const c = Number(await kv.ttl(k1));
}
