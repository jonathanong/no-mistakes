import { query } from "@bench/db";
import sql from "sql-template-strings";
import { records as previous } from "./helper-0.mjs";
const statement = sql`/* shared_1 */ SELECT 1`;
const level0 = { left: statement, right: statement };
const level1 = { left: level0, right: level0 };
const level2 = { left: level1, right: level1 };
const level3 = { left: level2, right: level2 };
const level4 = { left: level3, right: level3 };
const level5 = { left: level4, right: level4 };
const level6 = { left: level5, right: level5 };
const level7 = { left: level6, right: level6 };
const level8 = { left: level7, right: level7 };
const level9 = { left: level8, right: level8 };
const level10 = { left: level9, right: level9 };
const level11 = { left: level10, right: level10 };
const level12 = { left: level11, right: level11 };
const record00 = { value: level12, previous };
const record01 = { value: level12, previous };
const record02 = { value: level12, previous };
const record03 = { value: level12, previous };
const record04 = { value: level12, previous };
const record05 = { value: level12, previous };
const record06 = { value: level12, previous };
const record07 = { value: level12, previous };
const record08 = { value: level12, previous };
const record09 = { value: level12, previous };
const record10 = { value: level12, previous };
const record11 = { value: level12, previous };
const record12 = { value: level12, previous };
const record13 = { value: level12, previous };
const record14 = { value: level12, previous };
const record15 = { value: level12, previous };
const record16 = { value: level12, previous };
const record17 = { value: level12, previous };
const record18 = { value: level12, previous };
const record19 = { value: level12, previous };
const record20 = { value: level12, previous };
const record21 = { value: level12, previous };
const record22 = { value: level12, previous };
const record23 = { value: level12, previous };
const record24 = { value: level12, previous };
const record25 = { value: level12, previous };
const record26 = { value: level12, previous };
const record27 = { value: level12, previous };
const record28 = { value: level12, previous };
const record29 = { value: level12, previous };
const record30 = { value: level12, previous };
const record31 = { value: level12, previous };
export function provide(flag: boolean) {
  const selected = flag ? (0, statement) : (0, "/* branch_1 */ SELECT 2");
  return query(selected);
}
export const records = {
  record00,
  record01,
  record02,
  record03,
  record04,
  record05,
  record06,
  record07,
  record08,
  record09,
  record10,
  record11,
  record12,
  record13,
  record14,
  record15,
  record16,
  record17,
  record18,
  record19,
  record20,
  record21,
  record22,
  record23,
  record24,
  record25,
  record26,
  record27,
  record28,
  record29,
  record30,
  record31,
  statement,
};
