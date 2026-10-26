import * as db from "./api";
const kept = (input) => db.createUser(input); // no-mistakes-disable-line ts-no-function-aliases -- stable external API
