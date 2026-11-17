import * as db from "@example/db";
import { runtimeSubpath, typeOnlyType, importedParenthesizedType, shadowedNamespace, union, parenthesized, type, value, mixed, wrongType, wrongValue, mixedModule, subpath, wrongName, nested, typeOnlyValue } from "./namespace-union-helper";

union(db.read);
parenthesized(db.read);
type(db.read);
value(db.read);
mixed(db.read);
wrongType(db.read);
wrongValue(db.read);
mixedModule(db.read);
subpath(db.read);
wrongName(db.read);
nested(db.read);
typeOnlyValue(db.read);
runtimeSubpath(db.read);
typeOnlyType(db.read);
importedParenthesizedType(db.read);
shadowedNamespace(db, db.read);
