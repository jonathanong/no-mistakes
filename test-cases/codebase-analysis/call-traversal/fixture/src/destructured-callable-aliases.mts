function objectTarget() {}
function arrayTarget() {}

const { run: objectAlias } = { run: objectTarget };
const [arrayAlias] = [arrayTarget];

export function callThroughDestructuredAliases() {
  objectAlias();
  arrayAlias();
}
