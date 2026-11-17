import { query } from '@example/db';

const nullRec = Object.freeze({
  vote: 'users',
  tag: 'posts',
} as const);
function getNull(name: string) {
  if (!Object.hasOwn(nullRec, name)) return null;
  return nullRec[name as keyof typeof nullRec];
}

const voidRec = Object.freeze({
  vote: 'users',
  tag: 'posts',
} as const);
function getVoid(name: string) {
  if (!Object.hasOwn(voidRec, name)) return void 0;
  return voidRec[name as keyof typeof voidRec];
}

const sideRec = Object.freeze({
  vote: 'users',
  tag: 'posts',
} as const);
function getSide(name: string) {
  if (!Object.hasOwn(sideRec, name)) throw other[name];
  if (!Object.hasOwn(sideRec, name)) throw bag.item[name];
  return sideRec[name as keyof typeof sideRec];
}

const gapRec = Object.freeze({
  vote: 'users',
  tag: 'posts',
} as const);
function getNotCall(name: string, flag: boolean) {
  if (!flag) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getExtra(name: string) {
  const local = 1;
  if (!Object.hasOwn(gapRec, name)) throw new Error(String(local));
  return gapRec[name as keyof typeof gapRec];
}
function getAnd(name: string) {
  if (typeof name !== 'string' && !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getEqual(name: string) {
  if (typeof name === 'string' || !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getBare(name: string) {
  if (name !== 'string' || !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getVoidType(name: string) {
  if (void name !== 'string' || !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getMemberType(name: string) {
  if (typeof gapRec.id !== 'string' || !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getOtherType(name: string, other: string) {
  if (typeof other !== 'string' || !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getIdentType(name: string, other: string) {
  if (typeof name !== other || !Object.hasOwn(gapRec, name)) throw new Error('no');
  return gapRec[name as keyof typeof gapRec];
}
function getBlock(name: string) {
  if (!Object.hasOwn(gapRec, name)) {
    const hidden = 1;
    throw new Error(String(hidden));
  }
  return gapRec[name as keyof typeof gapRec];
}
function getExpr(name: string) {
  if (!Object.hasOwn(gapRec, name)) 0;
  return gapRec[name as keyof typeof gapRec];
}

const emptyRec = Object.freeze({} as const);
function getEmpty(name: string) {
  if (!Object.hasOwn(emptyRec, name)) throw new Error('no');
  return emptyRec[name as keyof typeof emptyRec];
}

const wideRec = Object.freeze({
  k0: { table: 't0' },
  k1: { table: 't1' },
  k2: { table: 't2' },
  k3: { table: 't3' },
  k4: { table: 't4' },
  k5: { table: 't5' },
  k6: { table: 't6' },
  k7: { table: 't7' },
  k8: { table: 't8' },
  k9: { table: 't9' },
  k10: { table: 't10' },
  k11: { table: 't11' },
  k12: { table: 't12' },
  k13: { table: 't13' },
  k14: { table: 't14' },
  k15: { table: 't15' },
  k16: { table: 't16' }
} as const);
function getWide(name: string) {
  if (!Object.hasOwn(wideRec, name)) throw new Error('no');
  return wideRec[name as keyof typeof wideRec];
}

const manyRec = Object.freeze({
  k0: { table: 't0' },
  k1: { table: 't1' },
  k2: { table: 't2' },
  k3: { table: 't3' },
  k4: { table: 't4' },
  k5: { table: 't5' },
  k6: { table: 't6' },
  k7: { table: 't7' },
  k8: { table: 't8' },
  k9: { table: 't9' },
  k10: { table: 't10' },
  k11: { table: 't11' },
  k12: { table: 't12' },
  k13: { table: 't13' },
  k14: { table: 't14' },
  k15: { table: 't15' },
  k16: { table: 't16' },
  k17: { table: 't17' },
  k18: { table: 't18' },
  k19: { table: 't19' },
  k20: { table: 't20' },
  k21: { table: 't21' },
  k22: { table: 't22' },
  k23: { table: 't23' },
  k24: { table: 't24' },
  k25: { table: 't25' },
  k26: { table: 't26' },
  k27: { table: 't27' },
  k28: { table: 't28' },
  k29: { table: 't29' },
  k30: { table: 't30' },
  k31: { table: 't31' },
  k32: { table: 't32' },
  k33: { table: 't33' },
  k34: { table: 't34' },
  k35: { table: 't35' },
  k36: { table: 't36' },
  k37: { table: 't37' },
  k38: { table: 't38' },
  k39: { table: 't39' },
  k40: { table: 't40' },
  k41: { table: 't41' },
  k42: { table: 't42' },
  k43: { table: 't43' },
  k44: { table: 't44' },
  k45: { table: 't45' },
  k46: { table: 't46' },
  k47: { table: 't47' },
  k48: { table: 't48' },
  k49: { table: 't49' },
  k50: { table: 't50' },
  k51: { table: 't51' },
  k52: { table: 't52' },
  k53: { table: 't53' },
  k54: { table: 't54' },
  k55: { table: 't55' },
  k56: { table: 't56' },
  k57: { table: 't57' },
  k58: { table: 't58' },
  k59: { table: 't59' },
  k60: { table: 't60' },
  k61: { table: 't61' },
  k62: { table: 't62' },
  k63: { table: 't63' },
  k64: { table: 't64' }
} as const);
function getMany(name: string) {
  if (!Object.hasOwn(manyRec, name)) throw new Error('no');
  return manyRec[name as keyof typeof manyRec];
}

export function run(kind: string, flag: boolean, other: string) {
  query(`/* null-return */ DELETE FROM ${getNull(kind)} WHERE id = $1`);
  query(`/* void-return */ DELETE FROM ${getVoid(kind)} WHERE id = $1`);
  query(`/* side-member */ DELETE FROM ${getSide(kind)} WHERE id = $1`);
  query(`/* not-call */ DELETE FROM ${getNotCall(kind, flag)} WHERE id = $1`);
  query(`/* extra-stmt */ DELETE FROM ${getExtra(kind)} WHERE id = $1`);
  query(`/* and-guard */ DELETE FROM ${getAnd(kind)} WHERE id = $1`);
  query(`/* equal-typeof */ DELETE FROM ${getEqual(kind)} WHERE id = $1`);
  query(`/* bare-compare */ DELETE FROM ${getBare(kind)} WHERE id = $1`);
  query(`/* void-typeof */ DELETE FROM ${getVoidType(kind)} WHERE id = $1`);
  query(`/* member-typeof */ DELETE FROM ${getMemberType(kind)} WHERE id = $1`);
  query(`/* other-typeof */ DELETE FROM ${getOtherType(kind, other)} WHERE id = $1`);
  query(`/* ident-typeof */ DELETE FROM ${getIdentType(kind, other)} WHERE id = $1`);
  query(`/* bare-block */ DELETE FROM ${getBlock(kind)} WHERE id = $1`);
  query(`/* expr-guard */ DELETE FROM ${getExpr(kind)} WHERE id = $1`);
  query(`/* empty-record */ DELETE FROM ${getEmpty(kind)} WHERE id = $1`);
  query(`/* wide-record */ DELETE FROM ${getWide(kind).table} WHERE id = $1`);
  query(`/* many-records */ DELETE FROM ${getMany(kind).table} WHERE id = $1`);
}
