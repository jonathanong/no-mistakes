declare const original: string;

const localAlias = original;
export const exportedAlias = original;
const manyAliases = original,
  anotherAlias = original;

// Parentheses and non-null wrappers alone do not express a target type.
const parenthesizedAlias = ((original));
const wrappedNonNullAlias = ((original!)!);
