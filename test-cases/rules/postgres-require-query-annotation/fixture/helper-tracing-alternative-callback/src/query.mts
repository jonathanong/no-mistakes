import sql from 'sql-template-strings';
import { write } from '@app/db';

export function alternativeCallbackCanMutateCapture(condition: boolean) {
  const statement = sql`/* callback alternative */ SELECT 1`;
  const callback = condition ? (() => unknownMutation(statement)) : (() => 'unused');
  const ignored = unknownConsumer(callback);
  write(statement); // unanalyzable:alternative-callback-capture
}

export function returnedCallbackKeepsItsArmLocalFrame(condition: boolean) {
  const statement = sql`/* arm local callback */ SELECT 1`;
  function callbackFactory() {
    const local = statement;
    return () => unknownMutation(local);
  }
  const callback = condition ? callbackFactory() : (() => 'unused');
  const ignored = unknownConsumer(callback);
  write(statement); // unanalyzable:alternative-local-frame
}

export function conditionalSqlRemainsUnproven(condition: boolean) {
  write(condition ? '/* one */ SELECT 1' : '/* two */ SELECT 2'); // unanalyzable:conditional-sql
}
