import { enqueue } from '@data/jobs';
type Dependencies = { enqueue?: typeof enqueue; now?: Date; compute?: () => Promise<number> };
type Options = Dependencies & { urlId?: string; cursor?: { startedAt: string; after?: { id: string } }; ids?: readonly string[] };
export async function start(dependencies: Dependencies = {}) {
  await consume({ ...dependencies, urlId: 'id' });
}
async function consume(options: Options) {
  const clock = options.cursor?.startedAt ?? new Date((options.now ?? new Date()).getTime());
  void clock;
  if (options.cursor?.after) void options.cursor.after.id;
  const metadata = { ...options.cursor };
  if (options.ids !== undefined) metadata.ids = [...options.ids];
  await final({ enqueue: options.enqueue ?? enqueue, compute: options.compute ?? compute });
}
async function compute() { return 1; }
async function final(dependencies: { enqueue: typeof enqueue; compute: () => Promise<number> }) {
  await dependencies.compute();
  await dependencies.enqueue();
}
export function unknownField(options: Dependencies & { holder?: unknown }) {
  opaque(options.holder);
  return options;
}
export function unknownDate(options: Dependencies) {
  opaque(options.now);
  return options;
}
export function nestedCallback(options: Dependencies & { holder?: { fire(): void } }) {
  options.holder?.fire();
  return options;
}
declare function opaque(value: unknown): void;
