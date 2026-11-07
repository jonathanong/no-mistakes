import { read } from "db";

export async function finiteWrites(flag: boolean, ignored: string) {
  const record = { table: "alpha" };
  let templateValue = "alpha";
  templateValue = `rel_${1}`;
  await read(`SELECT * FROM ${templateValue}`);
  let additionValue = "alpha";
  additionValue = "rel_" + 2;
  await read(`SELECT * FROM ${additionValue}`);
  let conditionalValue = "alpha";
  conditionalValue = flag ? "alpha" : "beta";
  await read(`SELECT * FROM ${conditionalValue}`);
  let logicalValue = "alpha";
  logicalValue = "alpha" || "beta";
  await read(`SELECT * FROM ${logicalValue}`);
  let awaitedValue = "alpha";
  awaitedValue = await "alpha";
  await read(`SELECT * FROM ${awaitedValue}`);
  let staticValue = "alpha";
  staticValue = record.table;
  await read(`SELECT * FROM ${staticValue}`);
  let computedValue = "alpha";
  computedValue = record["table"];
  await read(`SELECT * FROM ${computedValue}`);
  let sequenceValue = "alpha";
  sequenceValue = (ignored, "alpha");
  await read(`SELECT * FROM ${sequenceValue}`);
  let conditionalWrite = "alpha";
  if (flag) conditionalWrite = "beta";
  await read(`SELECT * FROM ${conditionalWrite}`);
  let replacedValue = ignored;
  replacedValue = "alpha";
  await read(`SELECT * FROM ${replacedValue}`);
}
