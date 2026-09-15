// Unused `const Lazy = dynamic(() => import('./foo.mts'))` is still a
// string-literal `import()`, so `foo.mts` is reachable from this file's test.
import dynamic from "next/dynamic";

const Lazy = dynamic(() => import("./foo.mts"));

const wrap = (component: () => unknown) => component;

export default wrap(() => {
  const Lazy = 1;
  return Lazy;
});
