import * as db from "./api";
const reported = (input) => db.createUser(input);
// no-mistakes-disable-next-line ts-no-function-aliases -- stable external API
const kept = (input) => db.createUser(input);
