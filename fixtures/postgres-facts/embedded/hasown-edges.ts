import { query } from '@example/db';

function hasOwn(_record: object, _key: string) {
  return false;
}

const objectNs = { Object };

const cooked = Object.freeze({
  vote: `users`,
  tag: `posts`,
} as const);
function getCooked(name: string) {
  if (!Object.hasOwn(cooked, name)) throw new Error('no');
  return cooked[name as keyof typeof cooked];
}

const mixedKinds = Object.freeze({
  vote: 'users',
  tag: Object.freeze({ table: 'posts' }),
} as const);
function getMixedKinds(name: string) {
  if (!Object.hasOwn(mixedKinds, name)) throw new Error('no');
  return mixedKinds[name as keyof typeof mixedKinds];
}

const getterRec = Object.freeze({
  get vote() {
    return 'users';
  },
} as const);
function getGetter(name: string) {
  if (!Object.hasOwn(getterRec, name)) throw new Error('no');
  return getterRec[name as keyof typeof getterRec];
}

const emptyKey = Object.freeze({ '': 'users' } as const);
function getEmptyKey(name: string) {
  if (!Object.hasOwn(emptyKey, name)) throw new Error('no');
  return emptyKey[name as keyof typeof emptyKey];
}

const emptyNested = Object.freeze({ vote: Object.freeze({}) } as const);
function getEmptyNested(name: string) {
  if (!Object.hasOwn(emptyNested, name)) throw new Error('no');
  return emptyNested[name as keyof typeof emptyNested];
}

const duplicate = Object.freeze({
  vote: Object.freeze({ table: 'users' }),
  vote: Object.freeze({ table: 'posts' }),
} as const);
function getDuplicate(name: string) {
  if (!Object.hasOwn(duplicate, name)) throw new Error('no');
  return duplicate[name as keyof typeof duplicate];
}

const uneven = Object.freeze({
  vote: Object.freeze({ table: 'users', column: 'id' }),
  tag: Object.freeze({ table: 'posts' }),
} as const);
function getUneven(name: string) {
  if (!Object.hasOwn(uneven, name)) throw new Error('no');
  return uneven[name as keyof typeof uneven];
}

const blank = Object.freeze({ vote: '' } as const);
function getBlank(name: string) {
  if (!Object.hasOwn(blank, name)) throw new Error('no');
  return blank[name as keyof typeof blank];
}

const frozenNull = Object.freeze(null);
function getFrozenNull(name: string) {
  if (!Object.hasOwn(frozenNull, name)) throw new Error('no');
  return frozenNull[name as keyof typeof frozenNull];
}

const restRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getRest(name: string, ..._rest: string[]) {
  if (!Object.hasOwn(restRec, name)) throw new Error('no');
  return restRec[name as keyof typeof restRec];
}

const defaultRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getDefault(name = 'vote') {
  if (!Object.hasOwn(defaultRec, name)) throw new Error('no');
  return defaultRec[name as keyof typeof defaultRec];
}

const patternRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getPattern({ name }: { name: string }) {
  if (!Object.hasOwn(patternRec, name)) throw new Error('no');
  return patternRec[name as keyof typeof patternRec];
}

const typeofRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getTypeof(name: string) {
  if (typeof name) throw new Error('no');
  return typeofRec[name as keyof typeof typeofRec];
}

const plainRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getPlain(name: string) {
  if (!hasOwn(plainRec, name)) throw new Error('no');
  return plainRec[name as keyof typeof plainRec];
}

const objectNsRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getObjectNs(name: string) {
  if (!objectNs.Object.hasOwn(objectNsRec, name)) throw new Error('no');
  return objectNsRec[name as keyof typeof objectNsRec];
}

const stringArgRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getStringArg(name: string) {
  if (!Object.hasOwn(stringArgRec, 'name')) throw new Error('no');
  return stringArgRec[name as keyof typeof stringArgRec];
}

const mismatch = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getMismatch(name: string, other: string) {
  if (!Object.hasOwn(mismatch, name)) throw new Error('no');
  return mismatch[other as keyof typeof mismatch];
}

const statRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getStatic(name: string) {
  if (!Object.hasOwn(statRec, name)) throw new Error('no');
  return statRec.vote;
}

const ns = { stat: { vote: 'users' } };
const statMember = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getNsMember(name: string) {
  if (!Object.hasOwn(statMember, name)) throw new Error('no');
  return ns.stat[name];
}

const literalKeyRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getLiteralKey(name: string) {
  if (!Object.hasOwn(literalKeyRec, name)) throw new Error('no');
  return literalKeyRec['vote'];
}

const optionalRec = Object.freeze({ vote: 'users', tag: 'posts' } as const);
function getOptional(name: string) {
  if (!Object.hasOwn(optionalRec, name)) throw new Error('no');
  return optionalRec?.[name];
}

Object.hasOwn(1, 'x');

export function run(kind: string) {
  query(`/* cooked */ DELETE FROM ${getCooked(kind)} WHERE id = $1`);
  query(`/* mixed-kinds */ DELETE FROM ${getMixedKinds(kind)} WHERE id = $1`);
  query(`/* getter */ DELETE FROM ${getGetter(kind)} WHERE id = $1`);
  query(`/* empty-key */ DELETE FROM ${getEmptyKey(kind)} WHERE id = $1`);
  query(`/* empty-nested */ DELETE FROM ${getEmptyNested(kind)} WHERE id = $1`);
  query(`/* duplicate */ DELETE FROM ${getDuplicate(kind)} WHERE id = $1`);
  query(`/* uneven */ DELETE FROM ${getUneven(kind)} WHERE id = $1`);
  query(`/* blank */ DELETE FROM ${getBlank(kind)} WHERE id = $1`);
  query(`/* frozen-null */ DELETE FROM ${getFrozenNull(kind)} WHERE id = $1`);
  query(`/* rest */ DELETE FROM ${getRest(kind)} WHERE id = $1`);
  query(`/* default */ DELETE FROM ${getDefault()} WHERE id = $1`);
  query(`/* pattern */ DELETE FROM ${getPattern({ name: kind })} WHERE id = $1`);
  query(`/* typeof */ DELETE FROM ${getTypeof(kind)} WHERE id = $1`);
  query(`/* plain */ DELETE FROM ${getPlain(kind)} WHERE id = $1`);
  query(`/* object-ns */ DELETE FROM ${getObjectNs(kind)} WHERE id = $1`);
  query(`/* string-arg */ DELETE FROM ${getStringArg(kind)} WHERE id = $1`);
  query(`/* mismatch */ DELETE FROM ${getMismatch(kind, kind)} WHERE id = $1`);
  query(`/* static */ DELETE FROM ${getStatic(kind)} WHERE id = $1`);
  query(`/* ns-member */ DELETE FROM ${getNsMember(kind)} WHERE id = $1`);
  query(`/* literal-key */ DELETE FROM ${getLiteralKey(kind)} WHERE id = $1`);
  query(`/* optional-return */ DELETE FROM ${getOptional(kind)} WHERE id = $1`);
}
