const data = { timeout: 1 };

// The tail cannot move index 0, including through an immutable array alias.
const knownPrefix = [data, ...externalArray];
const copiedPrefix = [...knownPrefix, ...externalTail];
copiedPrefix[0].timeout = true;

// Known entries and offsets before an opaque tail stay addressable.
const knownOffset = [null, data, ...externalArray];
knownOffset[1].timeout = true;
const completePrefix = [data];
const missingAfterKnownSpread = [...completePrefix];
missingAfterKnownSpread[9].timeout = false;
const aliasPrefix = [data];
const spreadAlias = [...aliasPrefix, ...externalTail];
spreadAlias[0].timeout = true;

// An incomplete nested spread has no proven total length for later elements.
const incompleteInner = [data, ...externalArray];
const incompleteNested = [...incompleteInner, data];
incompleteNested[0].timeout = true;
incompleteNested[1].timeout = false;

// Earlier unknown offsets and holes remain unproved.
const unknownFirst = [...externalArray, data];
unknownFirst[0].timeout = false;
// The hole intentionally has no own descriptor at index 0.
// oxlint-disable-next-line no-sparse-arrays
const sparse = [, data, ...externalTail];
sparse[0].timeout = false;
const customIterable = [...externalIterable, data];
customIterable[0].timeout = false;
