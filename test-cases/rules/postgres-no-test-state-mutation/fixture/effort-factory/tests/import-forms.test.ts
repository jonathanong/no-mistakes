import { beginTransaction as namedFactory } from '../db/index';
import * as factoryNamespace from '../db/index';
import defaultFactory from '../db/index';

export async function run() {
  const namedTx = await namedFactory();
  await namedTx('DROP TABLE app.factory_named_control'); // named import is selected

  const namespaceTx = await factoryNamespace.beginTransaction();
  await namespaceTx('DROP TABLE app.factory_namespace'); // namespace form is outside selection

  const defaultTx = await defaultFactory();
  await defaultTx('DROP TABLE app.factory_default'); // default form is outside selection
}
