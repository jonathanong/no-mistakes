import { read } from "db";

export async function scalarWrites(input: string, flag: boolean) {
  let templateValue = "alpha";
  templateValue = `rel_${input}`;
  await read(`SELECT * FROM ${templateValue}`);
  let additionValue = "alpha";
  additionValue = "rel_" + input;
  await read(`SELECT * FROM ${additionValue}`);
  let conditionalValue = "alpha";
  conditionalValue = flag ? input : "alpha";
  await read(`SELECT * FROM ${conditionalValue}`);
  let logicalValue = "alpha";
  logicalValue = input || "alpha";
  await read(`SELECT * FROM ${logicalValue}`);
  let awaitedValue = "alpha";
  awaitedValue = await input;
  await read(`SELECT * FROM ${awaitedValue}`);
  let staticValue = "alpha";
  staticValue = process.env.TABLE!;
  await read(`SELECT * FROM ${staticValue}`);
  let computedValue = "alpha";
  computedValue = process.env["TABLE"]!;
  await read(`SELECT * FROM ${computedValue}`);
  let sequenceValue = "alpha";
  sequenceValue = (0, input);
  await read(`SELECT * FROM ${sequenceValue}`);
  let conditionalWrite = input;
  if (flag) conditionalWrite = "alpha";
  await read(`SELECT * FROM ${conditionalWrite}`);
}
