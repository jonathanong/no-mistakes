import { write } from '@app/db';
function create(statement: string) {
  const reader = () => write(statement);
  return () => {
    // A callback consumed in only one sibling remains available after the join.
    const chosen = flag ? opaque(reader) : mutate(statement);
    const after = opaque(reader);
  };
}
const installer = create('/* initial */ SELECT 1');
const consumed = opaque(installer);
