declare const db: { createUser(input: string): string };
const reported = (input: string) => db.createUser(input);
// no-mistakes-disable-next-line ts-no-function-aliases -- stable external API
const kept = (input: string) => db.createUser(input);
