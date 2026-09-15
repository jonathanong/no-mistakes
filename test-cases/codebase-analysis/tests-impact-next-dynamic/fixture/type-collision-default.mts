// The unused value `const Lazy = dynamic(() => import('./foo.mts'))` is still a
// string-literal `import()`, so `foo.mts` is reachable from this file's test.
import dynamic from "next/dynamic";

type Lazy = Record<string, never>;
const Lazy = dynamic(() => import("./foo.mts"));

export default {} as Lazy;
