export function called() { return 1; }
called();
export const read = 2;
const result = read;
export const onlyExport = 3;
const listed = 4;
export { listed };
const aliased = 5;
export { aliased as publicAlias };
const usedAlias = 6;
export { usedAlias as publicUsedAlias };
const useAlias = usedAlias;
export const shadowed = 7;
function inner(shadowed: number) { return shadowed; }
export function recursive() { return recursive(); }
export const typeQuery = 8;
type Query = typeof typeQuery;
export let pureWrite = 0;
pureWrite = 1;
export let readWrite = 0;
readWrite += 1;
