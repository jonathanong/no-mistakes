function parameterOverwrite(input: string) {
  probe(input); // open: true
  input = "fixed";
  probe(input); // open: true
}
function boundedAliases() {
  let first = "fixed";
  let second = first;
  first = second;
  probe(first); // open: false
}
function numericOverwrite(input: number) {
  let entry = input;
  entry = input;
  probe(entry); // open: false
}
function conditionalWrite(input: string, flag: boolean) {
  let entry = "fixed";
  if (flag) entry = input;
  probe(entry); // open: true
}
