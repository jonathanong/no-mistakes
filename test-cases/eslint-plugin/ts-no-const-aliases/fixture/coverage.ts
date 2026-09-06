declare function original<T = string>(): T;

const assertedAlias = original as unknown as typeof original;
const assertionAlias = <typeof original>original;
const nonNullAlias = original!;
const satisfiedAlias = original satisfies typeof original;
const instantiatedAlias = original<string>;
const allWrappedAlias = (original as typeof original)! satisfies typeof original;
const optionalMemberAlias = original?.name;
