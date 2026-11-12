import type { TransactionQuery } from './database.mts';
export const MUTATED = [{ run: (query: TransactionQuery) => query('mutated', []) }];
MUTATED[0].run = async () => ({ rowCount: 0 });
export const ESCAPED = [{ run: (query: TransactionQuery) => query('escaped', []) }];
declare function escape(value: unknown): void;
escape(ESCAPED);
export const IMPORTER_MUTATED = [{ run: (query: TransactionQuery) => query('importer mutated', []) }];
export const IMPORTER_ESCAPED = [{ run: (query: TransactionQuery) => query('importer escaped', []) }];
declare const unknownRun: (query: TransactionQuery) => Promise<unknown>;
export const UNKNOWN = [{ run: unknownRun }];
export const RECEIVER = [{
  run: (query: TransactionQuery) => query('receiver', []),
  check: function() { this.run = async () => ({ rowCount: 0 }); },
}];
export const SIBLING_MUTATED = [{ run: (query: TransactionQuery) => query('sibling mutated', []) }];
export const SIBLING_ESCAPED = [{ run: (query: TransactionQuery) => query('sibling escaped', []) }];
export const BARREL_MUTATED = [{ run: (query: TransactionQuery) => query('barrel mutated', []) }];
