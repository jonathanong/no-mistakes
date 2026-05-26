import { expect, test } from 'vitest';
import { format } from './index.mts';

test('format', () => {
    expect(format(1)).toBe('1');
});
