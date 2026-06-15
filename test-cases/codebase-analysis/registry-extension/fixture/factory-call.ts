import { makeFoo } from "./factories/foo";
import { makeBar } from "./factories/bar";
import { registry } from "./registry";

// Imported factory-call registrants (argument is a CallExpression).
registry.register(makeFoo());
registry.register(makeBar());
