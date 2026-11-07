export function summarize(input: string) {
  let candidate = `partition_${input}`;
  candidate = `final_${input}`;
  let idle = `unused_${input}`;
  let counter = 0;
  counter++;
  const statement = `SELECT * FROM ${candidate}`;
  return [statement, idle, counter];
}
