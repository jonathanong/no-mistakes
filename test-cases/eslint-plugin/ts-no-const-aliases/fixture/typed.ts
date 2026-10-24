declare const original: unknown;
type Config = { enabled: boolean };

// These bindings express type intent even though the runtime value is unchanged.
const asserted = original as Config;
const angleAsserted = <Config>original;
const checked = original satisfies Config;
const annotated: Config = original;
export const exportedAnnotated: Config = original;
const wrappedAsserted = ((original as Config)!)!;
const wrappedAngleAsserted = ((<Config>original)!)!;
const wrappedChecked = ((original satisfies Config)!)!;
const assertedNonNull = (original!) as Config;
const checkedNonNull = (original!) satisfies Config;
const annotatedNonNull: Config = (original!);
const firstTyped: Config = original, secondTyped: Config = original;
