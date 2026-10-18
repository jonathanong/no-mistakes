import { beginTransaction } from '@example/db';
function shadowed(Symbol: { asyncDispose: string }) {
  const transaction = beginTransaction();
  // The shadowed key may equal `query`; retain only the callable proof.
  transaction[Symbol.asyncDispose] = (_request: object) => undefined;
  transaction.query({ resource: 'http' });
  transaction('SELECT direct computed');
}
