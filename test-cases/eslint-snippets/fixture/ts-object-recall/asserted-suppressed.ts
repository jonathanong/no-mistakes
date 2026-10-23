import { db } from "./api";
type Handler = typeof db.createUser;
// These identity casts report without their suppression directives.
// no-mistakes-disable-next-line ts-no-function-aliases -- reviewed compatibility alias
const asserted = input => (db.createUser as Handler)(input);
const sameLine = input => (db.createUser as Handler)(input); // no-mistakes-disable-line ts-no-function-aliases -- reviewed compatibility alias
