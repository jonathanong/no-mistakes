// Production-only imports must not satisfy the test rule's executor-name audit.
import { openTransaction } from '@example/db';
export const begin = openTransaction;
