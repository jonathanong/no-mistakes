import { importedTable } from "./assignment-export.ts";

function identity(value: string) { return value; }

export async function assignmentFacts(input: string, flag: boolean, key: string) {
  let callValue: unknown = "alpha";
  callValue = identity(input);
  let awaitedCallValue: unknown = "alpha";
  awaitedCallValue = await identity(input);
  let objectValue: unknown = "alpha";
  objectValue = { nested: { rows: [input, input, input] } };
  let arrayValue: unknown = "alpha";
  arrayValue = [input, { nested: [input, input] }];
  let jsxValue: unknown = "alpha";
  jsxValue = <section data-table={input}><span>{input}</span></section>;
  let fragmentValue: unknown = "alpha";
  fragmentValue = <><span>{input}</span></>;
  let arrowValue: unknown = "alpha";
  arrowValue = () => ({ nested: [input, input] });
  let functionValue: unknown = "alpha";
  functionValue = function () { return { nested: [input, input] }; };
  let mixedTemplate: unknown = "alpha";
  mixedTemplate = `rel_${{ nested: [input, input] }}_${input}`;
  let mixedAddition: unknown = "alpha";
  mixedAddition = input + [input, { nested: input }];
  let mixedConditional: unknown = "alpha";
  mixedConditional = flag ? { nested: [input, input] } : input;
  let projectedObject: unknown = "alpha";
  projectedObject = ({ table: input }).table;
  let projectedArray: unknown = "alpha";
  projectedArray = [input][0];
  let dynamicValue: unknown = "alpha";
  dynamicValue = process.env[key];
  let sequencePayload: unknown = "alpha";
  sequencePayload = (input, { nested: [input, input] });
  importedTable = input;
  unboundTable = input;
  let destructured = "alpha";
  ({ table: destructured } = { table: input });
  let compound = 1;
  compound += 1;
  let opaqueInitial: unknown = { nested: [input, input] };
  opaqueInitial = input;
  let unwritten = input;
}
