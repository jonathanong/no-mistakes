// no-mistakes-disable-file ts-no-function-aliases -- reviewed compatibility aliases
import { db } from "./api";
type Handler = typeof db.createUser;
// This identity cast reports without the file directive.
const asserted = input => (db.createUser as Handler)(input);
