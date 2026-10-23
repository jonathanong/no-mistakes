// no-mistakes-disable-next-line ts-no-function-aliases -- reviewed compatibility alias
const asserted = (input: Input) => (db.createUser as Handler)(input);
const applied = (input: Input) => (createUser as Handler).apply(undefined, [input]); // no-mistakes-disable-line ts-no-function-aliases -- reviewed compatibility alias
