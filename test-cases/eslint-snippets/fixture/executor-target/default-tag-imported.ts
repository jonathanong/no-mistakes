import { readPool } from "@example/psql";
import sql from "sql-template-strings";
import {
  hiddenDefaultTag,
  hiddenDestructuredLocal,
  hiddenDestructuredParameter,
  hiddenNested,
  fragmentMethod,
  scalarMethod,
  renamedField,
  multiField,
  defaultField,
  computedField,
  mutatedField,
  mutatedAliasField,
  assignedField,
  castAssignedField,
  castAliasField,
  boxedField,
  methodField,
  repeatedField,
  scalarFieldMethod,
  optionBag,
  nestedMember,
  directNested,
  writtenDestructured,
  writtenLocal,
  computedAlias,
  loopDestructured,
  arrayElision,
  repeatedInterpolation,
  quotedField,
  uninitializedScalar,
  nestedParameter,
  twoQueries,
  differentReceiver,
  visibleDefaultTag,
} from "./default-tag-helper";

visibleDefaultTag("west", readPool);
hiddenDefaultTag(sql`partition = 'west'`, readPool);
hiddenDestructuredParameter({ where: sql`partition = 'west'` }, readPool);
hiddenDestructuredLocal({ where: sql`partition = 'west'` }, readPool);
hiddenDestructuredParameter({ where: 1 }, readPool);
hiddenDestructuredLocal({ where: 1 }, readPool);
hiddenNested({ where: { sql: sql`id = 1` }, sql: 1 }, readPool);
scalarMethod("west", readPool);
fragmentMethod(sql`id = 1`, readPool);
const scalarOptions = { where: 1 };
const scalarOptionsBase = { where: 1 };
const scalarOptionsAlias = scalarOptionsBase;
hiddenDestructuredParameter(scalarOptionsAlias, readPool);
hiddenDestructuredLocal(scalarOptions, readPool);
const spreadOptions = { where: 1 };
hiddenDestructuredParameter({ ...spreadOptions }, readPool);
hiddenDestructuredParameter({ ["where"]: 1 }, readPool);
hiddenDestructuredParameter({ other: 1 }, readPool);
hiddenDestructuredParameter(opaqueOptions, readPool);
renamedField({ where: sql`id = 1` }, readPool);
renamedField({ where: 1 }, readPool);
multiField({ noise: 1, where: sql`id = 1` }, readPool);
multiField({ noise: 1, where: 1 }, readPool);
defaultField({}, readPool);
computedField({ where: sql`id = 1` }, readPool);
const mutatedOptions = { where: 1 };
mutatedOptions.where = sql`id = 1`;
hiddenDestructuredParameter(mutatedOptions, readPool);
mutatedField({ where: 1 }, sql`id = 1`, readPool);
mutatedAliasField({ where: 1 }, sql`id = 1`, readPool);
assignedField({ where: 1 }, sql`id = 1`, readPool);
castAssignedField({ where: 1 }, sql`id = 1`, readPool);
castAliasField({ where: 1 }, sql`id = 1`, readPool);
boxedField({ where: 1 }, sql`id = 1`, readPool);
methodField(
  {
    where: 1,
    mutate() {
      this.where = sql`id = 1`;
    },
  },
  readPool,
);
repeatedField({ where: 1 }, readPool);
scalarFieldMethod({ where: 1 }, readPool);
optionBag({ query: readPool, where: 1 });
optionBag({ query: readPool, where: sql`id = 1` });
const reflectedOptions = { where: 1 };
Object.assign(reflectedOptions, { where: sql`id = 1` });
hiddenDestructuredParameter(reflectedOptions, readPool);
const aliasedOptions = { where: 1 };
const mutableAlias = aliasedOptions;
mutableAlias.where = sql`id = 1`;
hiddenDestructuredParameter(aliasedOptions, readPool);
nestedMember({ inner: { where: sql`id = 1` }, where: 1 }, readPool);
directNested({ where: { sql: sql`id = 1` } }, readPool);
writtenDestructured({ where: 1 }, sql`id = 1`, readPool);
writtenLocal({ where: 1 }, sql`id = 1`, readPool);
computedAlias({ where: sql`id = 1` }, readPool);
hiddenDestructuredParameter({ where: 1 }, readPool);
hiddenDestructuredParameter(makeOptions(), readPool);
hiddenDestructuredParameter({ where: 1, where: sql`id = 1` }, readPool);
loopDestructured([{ where: sql`id = 1` }], readPool);
arrayElision([1, sql`id = 1`], readPool);
repeatedInterpolation(1, readPool);
quotedField({ "where-key": 1 }, readPool);
uninitializedScalar(readPool);
nestedParameter(readPool);
twoQueries({ query: readPool, where: 1 });
twoQueries({ query: readPool, where: sql`id = 1` });
differentReceiver(
  {
    where: 1,
    query() {
      this.where = sql`id = 1`;
    },
  },
  { query: readPool },
);
