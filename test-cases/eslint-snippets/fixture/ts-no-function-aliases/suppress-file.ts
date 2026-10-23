/* no-mistakes-disable-file ts-no-function-aliases */
declare const db: { createUser(input: string): string };
const kept = (input: string) => db.createUser(input);
