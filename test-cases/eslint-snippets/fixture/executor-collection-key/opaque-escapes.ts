import { query, type QueryExecutor } from "@example/db";

declare const mutator: { touch(value: unknown): void };
declare function unknownHelper(value: any): void;
const direct = new Map<QueryExecutor, unknown>();
mutator.touch(direct);
direct.get(query);

const nested = new Map<QueryExecutor, unknown>();
mutator.touch({ nested: [nested] });
nested.get(query);

const unrelated = new Map<QueryExecutor, unknown>();
declare const data: any;
unknownHelper(data);
unrelated.get(query);

const original = new Map<QueryExecutor, unknown>();
const alias = original;
mutator.touch(original);
alias.get(query);

const wrapped = new Map<QueryExecutor, unknown>();
mutator.touch((wrapped));
wrapped.get(query);

const shorthand = new Map<QueryExecutor, unknown>();
mutator.touch({ shorthand });
shorthand.get(query);

const spread = new Map<QueryExecutor, unknown>();
mutator.touch({ ...spread });
spread.get(query);

const conditional = new Map<QueryExecutor, unknown>();
mutator.touch(true ? conditional : conditional);
conditional.get(query);
