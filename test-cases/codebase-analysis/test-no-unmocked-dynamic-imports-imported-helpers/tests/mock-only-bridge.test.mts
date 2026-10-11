import { vi, test } from 'vitest';
vi.mock('./bridge.mts', () => ({ ready: true }));
import './bridge.mts';
import { loader } from '../src/bridge-loader.mts';
test('mocked bridge does not execute its helper', () => loader());
