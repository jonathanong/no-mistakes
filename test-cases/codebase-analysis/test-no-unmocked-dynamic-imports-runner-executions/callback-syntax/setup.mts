import { vi } from 'vitest';
// This colliding config-directory setup must never replace web/setup.mts.
vi.mock('./web/src/root-target.mts');
