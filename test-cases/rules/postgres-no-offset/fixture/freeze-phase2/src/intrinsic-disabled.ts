// No configured executor/type/factory source exists in this file.
// Shorthand assignment reaches the disabled global-write guard directly.
declare const replacement: { Object: typeof Object };
({ Object } = replacement); // NON_SQL
Object = replacement.Object; // NON_SQL
const metadata = Object.freeze({ hostname: "local" }); // NON_SQL
const hostname = metadata.hostname; // NON_SQL
