export let prefix = "/* branch */ SELECT 1";
if (flag) prefix = "/* other */ SELECT 2";
export function queryText() {
  return prefix;
}
