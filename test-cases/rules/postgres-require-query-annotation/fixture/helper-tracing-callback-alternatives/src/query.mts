import { write } from '@app/db';
function create(parameter: string) {
  const reader = () => write(arguments[0]);
  return () => {
    // Each sibling invokes the same callback against its own lexical slots.
    const chosen = flag
      ? ((arguments[0] = '/* annotated branch */ SELECT 1'), opaque(reader))
      : ((arguments[0] = 'SELECT 1'), opaque(reader));
  };
}
const installer = create('/* initial */ SELECT 1');
const consumed = opaque(installer);
