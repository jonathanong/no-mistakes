/* no-mistakes-disable-file ts-no-function-aliases */
import * as db from "./api";
const kept = (input) => db.createUser(input);
