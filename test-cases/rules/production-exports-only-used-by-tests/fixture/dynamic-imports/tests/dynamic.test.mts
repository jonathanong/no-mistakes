import { thenMember } from "../src/then-member.mjs";
import { thenDestructure } from "../src/then-destructure.mjs";
import LoaderDefault from "../src/loader-default.mjs";
import { awaitMember } from "../src/await-member.mjs";
import { awaitDestructure } from "../src/await-destructure.mjs";
import { unknownForm } from "../src/unknown.mjs";
import { present } from "../src/absent-target.mjs";
import { shared } from "../src/mixed-line-target.mjs";
export const used = [thenMember, thenDestructure, LoaderDefault, awaitMember, awaitDestructure, unknownForm, present, shared];
