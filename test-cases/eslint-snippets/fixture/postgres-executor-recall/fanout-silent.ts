import { read } from "@example/db";
import * as db from "@example/db";
import * as other from "@other/db";
const key = "read";
let mutable = read;
const { [key]: computed } = db;
const { read: defaulted = read } = db;
const { read: foreign } = other;
Promise.all(items.map(mutable));
Promise.all(items.map(computed));
Promise.all(items.map(defaulted));
Promise.all(items.map(foreign));
Promise.all(items.map((read) => read("SELECT 1")));
Promise.all(items.map(() => read.apply(null, args)));
Promise.all(items.map(() => read.call(...args)));
Promise.all(items.map(other.read));
Promise.all(items.map(db[key]));
// Passing Function.prototype.call loses its receiver; it is not invoking read.call.
Promise.all(items.map(read.call));
Promise.all(items.map(read["call"]));
Promise.all(items.map(read.apply));
// A type-only namespace supplies no value executor, even when used in invalid TS.
import type * as typedDb from "@example/db";
const { read: typedRead } = typedDb;
Promise.all(items.map(typedDb.read));
Promise.all(items.map(typedRead));
