import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";

const asyncRecord = Object.freeze({ vote: "users", post: "posts" } as const);
const generatorRecord = Object.freeze({ vote: "users", post: "posts" } as const);
const arrowRecord = Object.freeze({ vote: "users", post: "posts" } as const);
const expressionRecord = Object.freeze({ vote: "users", post: "posts" } as const);

async function asyncHelper(key: keyof typeof asyncRecord) {
  if (!Object.hasOwn(asyncRecord, key)) throw new Error("unknown table");
  return asyncRecord[key];
}

function* generatorHelper(key: keyof typeof generatorRecord) {
  if (!Object.hasOwn(generatorRecord, key)) throw new Error("unknown table");
  return generatorRecord[key];
}

const asyncArrow = async (key: keyof typeof arrowRecord) => {
  if (!Object.hasOwn(arrowRecord, key)) throw new Error("unknown table");
  return arrowRecord[key];
};

const asyncFunctionExpression = async function (key: keyof typeof expressionRecord) {
  if (!Object.hasOwn(expressionRecord, key)) throw new Error("unknown table");
  return expressionRecord[key];
};

export function run() {
  query(rawSql`SELECT id FROM ${asyncHelper("vote")} WHERE id = $1`);
  query(rawSql`SELECT id FROM ${generatorHelper("vote")} WHERE id = $1`);
  query(rawSql`SELECT id FROM ${asyncArrow("vote")} WHERE id = $1`);
  return query(rawSql`SELECT id FROM ${asyncFunctionExpression("vote")} WHERE id = $1`);
}
