// This intentionally checks syntactic namespace separation, not TypeScript diagnostics.
// A hoisted type name cannot introduce an earlier runtime import occurrence.
export function runtimeOnly() { return NonNullable("value"); }
import type { NonNullable } from './unselected.mts';
