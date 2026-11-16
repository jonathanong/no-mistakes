import { cache as kv } from '@app/cache';

async function conversions(k1: string, k2: string) {
  const a = Number(await kv.ttl(k1));
  const b = Boolean(await kv.exists(k2));
}

async function parsedValues(k1: string, k2: string) {
  const c = JSON.parse((await kv.get(k1)) ?? '{}');
  const d = (await kv.get(k2)) ?? null;
}

async function members(k1: string, k2: string) {
  const a = (await kv.get(k1)).value;
  const b = (await kv.get(k2)).value.toString();
}

async function assertions(k1: string, k2: string) {
  const a = (((await kv.get(k1)) as string));
  const b = String(((await kv.get(k2))!));
}

async function unconditionalLeft(k1: string, k2: string) {
  const a = (await kv.get(k1)) || 'empty';
  const b = (await kv.get(k2)) ?? 'empty';
}

async function unconditionalTest(k1: string, k2: string) {
  const a = (await kv.get(k1)) ? 'present' : 'absent';
  const b = (await kv.get(k2)) ? 'present' : 'absent';
}

async function returnedConversion(k1: string, k2: string) {
  const a = Number(await kv.ttl(k1));
  return Boolean(await kv.exists(k2));
}

async function assignedConversion(k1: string, k2: string) {
  let a, b;
  a = Number(await kv.ttl(k1));
  b = Boolean(await kv.exists(k2));
}

async function wrapperDependency(k1: string, k2: string) {
  const a = Number(await kv.ttl(k1));
  const b = combine(a, await kv.get(k2));
}

const first = await kv.get('first');
const second = await kv.get('second');

async function optionalResult(k1: string, k2: string) {
  const a = (await kv.get(k1))?.value;
  const b = (await kv.get(k2))?.value;
}

async function arrayHoles(k1: string, k2: string) {
  const a = [, await kv.get(k1)];
  const b = [await kv.get(k2), ,];
}
