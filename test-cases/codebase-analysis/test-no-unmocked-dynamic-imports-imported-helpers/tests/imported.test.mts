import { test } from 'vitest';
import './support.mts';
import { loader } from '../src/covered-loader.mts';
test('imported mock', () => loader());
