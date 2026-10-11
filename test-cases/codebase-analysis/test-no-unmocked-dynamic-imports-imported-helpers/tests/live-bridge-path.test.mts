import { vi, test } from 'vitest';
vi.mock('./bridge.mts', () => ({ ready: true }));
import './bridge.mts';
import './bridge-support.mts';
import { loader } from '../src/bridge-loader.mts';
test('direct helper path survives mocked bridge', () => loader());
