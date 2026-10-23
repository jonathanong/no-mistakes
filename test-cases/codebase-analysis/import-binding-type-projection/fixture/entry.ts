// Runtime import syntax is intentional: parameter types must borrow its binding.
import { Executor as RuntimeExecutor } from './types';
import type { Executor as DeclaredExecutor } from './types';
import { type Executor as InlineExecutor } from './types';

export function runtime(run: RuntimeExecutor) { run('select 1'); }
export function declared(run: DeclaredExecutor) { run('select 2'); }
export function inline(run: InlineExecutor) { run('select 3'); }
