import { query } from "@example/db";

/* Nine unrelated switch paths must retain the two pre-existing SQL values. */
let q = flag ? "SELECT 1" : "SELECT 2";
switch (mode) {
  case 0: break;
  case 1: break;
  case 2: break;
  case 3: break;
  case 4: break;
  case 5: break;
  case 6: break;
  case 7: break;
}
query(q);

/* Both unknown and known unrelated decisions preserve legacy single values. */
const plain = "SELECT 3";
query(plain);
switch (mode) {
  case 0: break;
  case 1: break;
  case 2: break;
  case 3: break;
  case 4: break;
  case 5: break;
  case 6: break;
  case 7: break;
}
query(plain);
switch (0) {
  case 0: break;
  case 1: break;
  case 2: break;
  case 3: break;
  case 4: break;
  case 5: break;
  case 6: break;
  case 7: break;
}
query(plain);

/* An unchanged value inside a fallthrough body also keeps its own choices. */
let fall = flag ? "SELECT 4" : "SELECT 5";
switch (mode) {
  case 0:
  case 1:
  case 2:
  case 3:
  case 4:
  case 5:
  case 6:
  case 7:
    query(fall);
    break;
}
query(fall);

/* Changed fields still share the selected path, including the unchanged arm. */
let prefix = "SELECT ";
let suffix = "0";
switch (mode) {
  case 0: prefix = "SELECT 6"; suffix = "1"; break;
  case 1: prefix = "SELECT 7"; suffix = "2"; break;
}
query(prefix + suffix);

/* Fallthrough must not combine a direct entry with another entry's prefix. */
let joinedPrefix = "SELECT ";
let joinedSuffix = "0";
switch (mode) {
  case 0: joinedPrefix = "SELECT 8";
  case 1: joinedSuffix = "3"; break;
  default: joinedPrefix = "SELECT 9"; joinedSuffix = "4";
}
query(joinedPrefix + joinedSuffix);

/* Sixteen changed statements remain within the cap; seventeen do not. */
let boundary = "SELECT 10";
switch (mode) {
  case 0: boundary = "SELECT 11"; break;
  case 1: boundary = "SELECT 12"; break;
  case 2: boundary = "SELECT 13"; break;
  case 3: boundary = "SELECT 14"; break;
  case 4: boundary = "SELECT 15"; break;
  case 5: boundary = "SELECT 16"; break;
  case 6: boundary = "SELECT 17"; break;
  case 7: boundary = "SELECT 18"; break;
  case 8: boundary = "SELECT 19"; break;
  case 9: boundary = "SELECT 20"; break;
  case 10: boundary = "SELECT 21"; break;
  case 11: boundary = "SELECT 22"; break;
  case 12: boundary = "SELECT 23"; break;
  case 13: boundary = "SELECT 24"; break;
  case 14: boundary = "SELECT 25"; break;
}
query(boundary);
let exceeded = "SELECT 26";
switch (mode) {
  case 0: exceeded = "SELECT 27"; break;
  case 1: exceeded = "SELECT 28"; break;
  case 2: exceeded = "SELECT 29"; break;
  case 3: exceeded = "SELECT 30"; break;
  case 4: exceeded = "SELECT 31"; break;
  case 5: exceeded = "SELECT 32"; break;
  case 6: exceeded = "SELECT 33"; break;
  case 7: exceeded = "SELECT 34"; break;
  case 8: exceeded = "SELECT 35"; break;
  case 9: exceeded = "SELECT 36"; break;
  case 10: exceeded = "SELECT 37"; break;
  case 11: exceeded = "SELECT 38"; break;
  case 12: exceeded = "SELECT 39"; break;
  case 13: exceeded = "SELECT 40"; break;
  case 14: exceeded = "SELECT 41"; break;
  case 15: exceeded = "SELECT 42"; break;
}
query(exceeded);

/* A genuinely opaque arm must remain opaque rather than lose its path. */
let unknown = "SELECT 43";
switch (mode) {
  case 0: unknown = externalSql; break;
  case 1: unknown = "SELECT 44"; break;
}
query(unknown);

/* A legacy dynamic single value does not become an enumerated call either. */
let legacy = "SELECT 45";
query(legacy);
switch (mode) {
  case 0: break;
  case 1: break;
  case 2: break;
  case 3: break;
  case 4: break;
  case 5: break;
  case 6: break;
  case 7: break;
}
query(legacy);
switch (0) {
  case 0: break;
  case 1: break;
  case 2: break;
  case 3: break;
  case 4: break;
  case 5: break;
  case 6: break;
  case 7: break;
}
query(legacy);
