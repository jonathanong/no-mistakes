import { db } from "./api";
type Handler = typeof db.createUser;
type Output = string;

// A typeof query names the exact reachable callable, so removing this cast keeps its type.
const assertedMember = (input) => (db.createUser as Handler)(input);
const angleMember = (input) => (<typeof db.createUser>db.createUser)(input);

export function createUser(input: string): string { return input; }
// The same-file target explicitly returns string; erasing this cast preserves the result type.
const assertedResult = (input) => (createUser(input) as Output);
const angleResult = (input) => (<string>createUser(input));

// These assertions carry contracts the file-local rule cannot prove equivalent.
const opaqueMember = (input) => (db.createUser as OtherHandler)(input);
const opaqueSatisfiedMember = (input) => (db.createUser satisfies OtherHandler)(input);
const opaqueResult = (input) => (createUser(input) as OtherOutput);
const satisfiedResult = (input) => createUser(input) satisfies OtherOutput;
const changedResult = (input) => (createUser(input) as number);

// Reachability and receiver identity remain necessary even for identity casts.
const catalog = { users: db };
const privateMember = (input) => (catalog.users.createUser as typeof catalog.users.createUser)(input);
const regexMember = (input) => (/a/.test as Handler)(input);
const callResult = (input) => (getClient().users.createUser as Handler)(input);
const wrongReceiver = (input) => (db.createUser as typeof db.createUser).call(otherDb, input);
const changedArgument = (input) => (db.createUser as typeof db.createUser)(transform(input));
const assertedReceiver = (input) => (db as typeof db).createUser(input);
const instantiated = (input) => (db.createUser<string>)(input);
