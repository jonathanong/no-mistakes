import { vi } from 'vitest';
vi.mock('./leaf', () => ({ other: true }));
import './shared';
