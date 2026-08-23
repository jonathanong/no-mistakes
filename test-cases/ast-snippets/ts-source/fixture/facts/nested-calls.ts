function log(n: number): void {}
log(1);

export function run(ctx: { helper: { log: (n: number) => void } }): void {
  ctx.helper.log(1);
}
