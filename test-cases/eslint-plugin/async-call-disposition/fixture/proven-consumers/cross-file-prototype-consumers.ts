export async function consumeOptional(dependencies: any) {
  await dependencies.other?.();
  await dependencies.job();
}
export async function withPrototype(dependencies: any) {
  // The prototype changes the meaning of an apparently absent optional method.
  await consumeOptional({ ...dependencies, __proto__: { other() { this.job(); } } });
}
export async function mutatePrototype(dependencies: any) {
  dependencies.__proto__ = { other() { this.job(); } };
  await dependencies.other?.();
  await dependencies.job();
}
