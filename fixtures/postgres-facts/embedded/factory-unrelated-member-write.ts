import { beginTransaction } from '@example/db';

const transaction = beginTransaction();
// Replacing a non-query property does not change the callable executor.
transaction[Symbol.asyncDispose] = async () => {};
transaction('SELECT 12');
