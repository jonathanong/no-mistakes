import { SIBLING_MUTATED, SIBLING_ESCAPED } from './control-stages.mts';
SIBLING_MUTATED[0].run = async () => ({ rowCount: 0 });
declare function escape(value: unknown): void;
escape(SIBLING_ESCAPED);
