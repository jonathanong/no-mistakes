import { vi } from 'vitest';
vi.mock('./broken', () => ({}));
import './broken';
export function run() { return 1; }
