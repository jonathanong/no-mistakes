import { it } from 'vitest';
import * as runner from 'vitest';

declare const unknownRunner: (rows: unknown) => (name: string, body: (row: string) => void) => void;
// The namespace import can replace the `.each` property on the named runner.
(runner.it as any).each = unknownRunner;
it.each(['safe', 'bad'] as const)('namespace mutation', row => {
  void row;
});
