declare const db: { createUser(input: string): string };
const kept = (input: string) => db.createUser(input); // no-mistakes-disable-line ts-no-function-aliases -- stable external API
