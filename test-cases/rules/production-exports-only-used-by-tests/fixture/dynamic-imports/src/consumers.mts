// Production uses every export below only through a dynamic import().
// Each form is statically provable, so none leaves an unknown selector.
declare function dynamic(loader: () => Promise<unknown>): unknown;

export const thenMember = import("./then-member.mjs").then((mod) => mod.thenMember);
export const thenDestructure = import("./then-destructure.mjs").then(({ thenDestructure }) => thenDestructure);
export const loaderDefault = dynamic(() => import("./loader-default.mjs"));

export async function load() {
  const ns = await import("./await-member.mjs");
  const { awaitDestructure } = await import("./await-destructure.mjs");
  return [ns.awaitMember, awaitDestructure];
}
