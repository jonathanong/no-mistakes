import { describe, expect, it } from 'vitest';
import { coreFn0 } from '@fixture/core';
describe('coreFn0', () => {
  it('returns a stable core value', () => {
    expect(coreFn0()).toContain('core-0');
  });
});
