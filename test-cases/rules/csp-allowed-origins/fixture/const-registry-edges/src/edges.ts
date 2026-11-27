// Odd on purpose so csp-allowed-origins covers const-registry edges.
import { REGISTERED_ORIGINS, type OriginName } from "./origins";

declare function setHeader(name: string, value: string): void;

export { type OriginName };

export function unusedBuilder() {}

export const { grouped } = { grouped: ["https://grouped.example.com"] };

const flagged = ["https://flagged.example.com"] as const;
flagged.prop = true;

const box = { key: 1 };
box["k"] = 2;

const leftFails = [missing + "tail"];
const rightFails = ["head" + missing];
void leftFails;
void rightFails;

const tags = ["a"];
void tags.map((tag) => tag).join(" ");

const nonce = "abc";
export const semi = "script-src 'self';'nonce-" + nonce + "'";
export const comma = "script-src 'self','nonce-" + nonce + "'";

void (true && false);

const foo = { bar: ["https://spread.example.com"] };
void ["connect-src", "'self'", ...foo.bar].join(" ");
void ["connect-src 'self'", , "img-src 'self'"].join("; ");

const unknownIdent = "dynamic";
setHeader("Content-Security-Policy", `connect-src 'self' ${unknownIdent}`);

export const groupedPolicy = ["connect-src", "'self'", ...grouped].join(" ");

const headerObject = {
  "Content-Security-Policy": `connect-src 'self' ${"https://static.example.com"} ${REGISTERED_ORIGINS.join(" ")}`,
};

setHeader(
  "Content-Security-Policy",
  `connect-src 'self' ${"https://static.example.com"} ${REGISTERED_ORIGINS.join(" ")}`,
);
setHeader("Content-Security-Policy", REGISTERED_ORIGINS.join(" "));

void headerObject;
