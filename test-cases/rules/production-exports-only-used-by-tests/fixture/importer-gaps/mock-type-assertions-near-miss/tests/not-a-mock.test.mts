export const load = async (original: () => Promise<unknown>) => ({
  ...((await original()) as typeof import("../src/not-a-mock.mjs")),
});
