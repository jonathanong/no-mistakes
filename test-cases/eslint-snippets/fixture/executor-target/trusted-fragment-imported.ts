import { read } from "@example/db";
import { sql } from "@example/sql";
import {
  hiddenAlias,
  hiddenDefaultFragment,
  hiddenParameter,
  hiddenReassigned,
  hiddenWrittenParameter,
  visibleDefaultScalar,
  visibleLetScalar,
  visibleScalar,
} from "./trusted-fragment-helper";

hiddenParameter(sql`id = 1`, read);
hiddenAlias(sql`id = 1`, read);
const predicate = sql`id = 1`;
hiddenParameter(predicate, read);
const typedPredicate: string = sql`id = 1`;
hiddenParameter(typedPredicate, read);
hiddenReassigned(sql`id = 1`, read);
hiddenWrittenParameter(1, sql`id = 1`, read);
hiddenDefaultFragment(read);
hiddenDefaultFragment(read, undefined);
hiddenDefaultFragment(read, true);
hiddenParameter(opaqueValue, read);
hiddenParameter(1, read);
hiddenAlias(2, read);
let scalarArg = 1;
hiddenParameter(scalarArg, read);
let mutableArg: unknown = 1;
mutableArg = sql`id = 1`;
hiddenParameter(mutableArg, read);
visibleScalar(1, read);
visibleDefaultScalar(read);
visibleLetScalar(read, 1);

type Scalar = number | string | null | undefined;
function callWithScalar(id: Scalar) {
  hiddenParameter(id, read);
}
callWithScalar(1);

function callWithMaybeUndefined(value: boolean | undefined) {
  hiddenDefaultFragment(read, value);
  visibleDefaultScalar(read, value);
}
callWithMaybeUndefined(undefined);
