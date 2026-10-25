import { DynamicConfig, Filter } from '@fixture/cache';
import { UsedError as SelectedError } from './errors.mjs';
new DynamicConfig({ key: 'primary', enabled: true });
new Filter<number>(42);
new SelectedError();
