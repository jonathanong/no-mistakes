import "./origins";
import type { REGISTERED_ORIGINS as TypeOrigins } from "./origins";
import defaultExport from "./origins";
import * as originNamespace from "./origins";
import { missingName } from "not-a-relative-module";

const [firstItem] = ["https://first.example.com"] as const;
void firstItem;

export const kept = ["https://kept.example.com"] as const;
export { kept as keptOrigins };

const sized = ["https://sized.example.com"] as const;
void sized.length;
void sized.join();

const sep = " ";
void sized.join(sep);

const emptyOrigins = [] as const;
void emptyOrigins.join(" ");

const optionalJoin = ["https://optional.example.com"] as const;
void optionalJoin?.join(" ");

const pushed = ["https://pushed.example.com"] as const;
pushed.push("https://extra.example.com");

const member = ["https://member.example.com"] as const;
member[0] = "https://changed.example.com";
member.flag = true;

let assignable = ["https://assignable.example.com"];
assignable = ["https://reassigned.example.com"];
[assignable] = [["https://pattern.example.com"]];

const plain = ["https://plain.example.com"] as const;
const alias = plain;

const holder = { values: ["https://holder.example.com"] };
void holder.values;
const copied = [...holder.values];
void copied;

const unknown: string[] = [];
void ["connect-src", ...unknown].join(" ");

void defaultExport;
void originNamespace;
void missingName;
void TypeOrigins;
