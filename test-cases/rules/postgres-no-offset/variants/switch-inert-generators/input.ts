import { query } from "@example/db";
declare function consume(...values: unknown[]): number;
declare function consumeTagged(strings: TemplateStringsArray, value: unknown): number;

/* direct generator creation. */
let guard1 = false;
switch (2) {
  case (function* () { guard1 = true; yield 1; })(): break;
  case 2: query(guard1 ? "SELECT 1 OFFSET 1" : "SELECT 1"); break; // pass
}

/* TypeScript-wrapped generator creation. */
let guard2 = false;
switch (2) {
  case ((function* () { guard2 = true; }) as () => Generator)(): break;
  case 2: query(guard2 ? "SELECT 2 OFFSET 1" : "SELECT 2"); break; // pass
}

/* async generator creation. */
let guard3 = false;
switch (2) {
  case (async function* () { guard3 = true; yield 1; })(): break;
  case 2: query(guard3 ? "SELECT 3 OFFSET 1" : "SELECT 3"); break; // pass
}

/* evaluated generator argument. */
let guard4 = false;
switch (2) {
  case (function* (value) { yield value; })(guard4 = true): break;
  case 2: query(guard4 ? "SELECT 4 OFFSET 1" : "SELECT 4"); break; // unanalyzable
}

/* generator parameter default. */
let guard5 = false;
switch (2) {
  case (function* (value = (guard5 = true)) { yield value; })(): break;
  case 2: query(guard5 ? "SELECT 5 OFFSET 1" : "SELECT 5"); break; // unanalyzable
}

/* ordinary invoked function. */
let guard6 = false;
switch (2) {
  case (function () { guard6 = true; return 1; })(): break;
  case 2: query(guard6 ? "SELECT 6 OFFSET 1" : "SELECT 6"); break; // unanalyzable
}

/* immediate next. */
let guard7 = false;
switch (2) {
  case (function* () { guard7 = true; yield 1; })().next(): break;
  case 2: query(guard7 ? "SELECT 7 OFFSET 1" : "SELECT 7"); break; // unanalyzable
}

/* for-of iteration. */
let guard8 = false;
switch (2) {
  case (() => { for (const value of (function* () { guard8 = true; yield 1; })()) {} return 1; })(): break;
  case 2: query(guard8 ? "SELECT 8 OFFSET 1" : "SELECT 8"); break; // unanalyzable
}

/* array spread consumes iterator. */
let guard9 = false;
switch (2) {
  case [...(function* () { guard9 = true; yield 1; })()]: break;
  case 2: query(guard9 ? "SELECT 9 OFFSET 1" : "SELECT 9"); break; // unanalyzable
}

/* advanced delegated generator. */
let guard10 = false;
switch (2) {
  case (function* () { yield* (function* () { guard10 = true; yield 1; })(); })().next(): break;
  case 2: query(guard10 ? "SELECT 10 OFFSET 1" : "SELECT 10"); break; // unanalyzable
}

/* iterator returned through arrow. */
let guard11 = false;
switch (2) {
  case (() => (function* () { guard11 = true; yield 1; })())().next(): break;
  case 2: query(guard11 ? "SELECT 11 OFFSET 1" : "SELECT 11"); break; // unanalyzable
}

/* returned iterator passed to unknown consumer. */
let guard12 = false;
switch (2) {
  case consume((() => (function* () { guard12 = true; yield 1; })())()): break;
  case 2: query(guard12 ? "SELECT 12 OFFSET 1" : "SELECT 12"); break; // unanalyzable
}

/* unused generator in ordinary arrow. */
let guard13 = false;
switch (2) {
  case (() => { (function* () { guard13 = true; })(); return 1; })(): break;
  case 2: query(guard13 ? "SELECT 13 OFFSET 1" : "SELECT 13"); break; // pass
}

/* for-in does not iterate. */
let guard14 = false;
switch (2) {
  case (() => { for (const key in (function* () { guard14 = true; yield 1; })()) {} return 1; })(): break;
  case 2: query(guard14 ? "SELECT 14 OFFSET 1" : "SELECT 14"); break; // pass
}

/* stored iterator is conservative. */
let guard15 = false;
switch (2) {
  case (() => { const iterator = (function* () { guard15 = true; yield 1; })(); iterator.next(); return 1; })(): break;
  case 2: query(guard15 ? "SELECT 15 OFFSET 1" : "SELECT 15"); break; // unanalyzable
}

/* stored iterator alias is conservative. */
let guard16 = false;
switch (2) {
  case (() => { const iterator = (function* () { guard16 = true; yield 1; })(); const alias = iterator; alias.next(); return 1; })(): break;
  case 2: query(guard16 ? "SELECT 16 OFFSET 1" : "SELECT 16"); break; // unanalyzable
}

/* stored unknown consumer is conservative. */
let guard17 = false;
switch (2) {
  case (() => { const iterator = (function* () { guard17 = true; yield 1; })(); consume(iterator); return 1; })(): break;
  case 2: query(guard17 ? "SELECT 17 OFFSET 1" : "SELECT 17"); break; // unanalyzable
}

/* stored unused iterator retains fallback. */
let guard18 = false;
switch (2) {
  case (() => { const iterator = (function* () { guard18 = true; yield 1; })(); return 1; })(): break;
  case 2: query(guard18 ? "SELECT 18 OFFSET 1" : "SELECT 18"); break; // unanalyzable
}

/* Existing literal SQL findings retain ordinary checking. */
let literalGuard = false;
switch (2) {
  case (function* () { literalGuard = true; })(): break;
  case 2: query("SELECT 19 OFFSET 1"); break; // offset
}

/* Actually advanced generator findings suppress on the executor line. */
let suppressedGuard = false;
switch (2) {
  case (function* () { suppressedGuard = true; yield 1; })().next(): break;
  case 2:
    // no-mistakes-disable-next-line postgres-no-offset
    query(suppressedGuard ? "SELECT 20 OFFSET 1" : "SELECT 20"); // suppressed
    break;
}

/* ordinary arrow parameter iterator default. */
let parameterGate21 = false;
switch (2) {
  case ((iterator = (function* () { parameterGate21 = true; yield 1; })()) => { iterator.next(); return 1; })(): break;
  case 2: query(parameterGate21 ? "SELECT 21 OFFSET 1" : "SELECT 21"); break; // unanalyzable
}

/* ordinary function parameter iterator default. */
let parameterGate22 = false;
switch (2) {
  case (function (iterator = (function* () { parameterGate22 = true; yield 1; })()) { iterator.next(); return 1; })(): break;
  case 2: query(parameterGate22 ? "SELECT 22 OFFSET 1" : "SELECT 22"); break; // unanalyzable
}

/* wrapped arrow parameter iterator default. */
let parameterGate23 = false;
switch (2) {
  case (((iterator = (function* () { parameterGate23 = true; yield 1; })()) => { iterator.next(); return 1; }) as () => number)(): break;
  case 2: query(parameterGate23 ? "SELECT 23 OFFSET 1" : "SELECT 23"); break; // unanalyzable
}

/* destructured parameter unknown iterator consumer. */
let parameterGate24 = false;
switch (2) {
  case (({ iterator = (function* () { parameterGate24 = true; yield 1; })() } = {}) => { consume(iterator); return 1; })(): break;
  case 2: query(parameterGate24 ? "SELECT 24 OFFSET 1" : "SELECT 24"); break; // unanalyzable
}

/* nested function parameter unknown iterator consumer. */
let parameterGate25 = false;
switch (2) {
  case (function ({ nested: { iterator = (function* () { parameterGate25 = true; yield 1; })() } = {} } = {}) { consume(iterator); return 1; })(): break;
  case 2: query(parameterGate25 ? "SELECT 25 OFFSET 1" : "SELECT 25"); break; // unanalyzable
}

/* array binding default iterator. */
let bindingGate26 = false;
switch (2) {
  case (() => { const [iterator = (function* () { bindingGate26 = true; yield 1; })()] = []; iterator.next(); return 1; })(): break;
  case 2: query(bindingGate26 ? "SELECT 26 OFFSET 1" : "SELECT 26"); break; // unanalyzable
}

/* object binding default iterator. */
let bindingGate27 = false;
switch (2) {
  case (() => { const { iterator = (function* () { bindingGate27 = true; yield 1; })() } = {}; consume(iterator); return 1; })(): break;
  case 2: query(bindingGate27 ? "SELECT 27 OFFSET 1" : "SELECT 27"); break; // unanalyzable
}

/* for-of binding default iterator. */
let bindingGate28 = false;
switch (2) {
  case (() => { for (const [iterator = (function* () { bindingGate28 = true; yield 1; })()] of [[]]) { iterator.next(); } return 1; })(): break;
  case 2: query(bindingGate28 ? "SELECT 28 OFFSET 1" : "SELECT 28"); break; // unanalyzable
}

/* catch binding default iterator. */
let bindingGate29 = false;
switch (2) {
  case (() => { try { throw {}; } catch ({ iterator = (function* () { bindingGate29 = true; yield 1; })() }) { iterator.next(); } return 1; })(): break;
  case 2: query(bindingGate29 ? "SELECT 29 OFFSET 1" : "SELECT 29"); break; // unanalyzable
}

/* argument spread consumes generator. */
let argumentGate30 = false;
switch (2) {
  case consume(...(function* () { argumentGate30 = true; yield 1; })()): break;
  case 2: query(argumentGate30 ? "SELECT 30 OFFSET 1" : "SELECT 30"); break; // unanalyzable
}

/* unknown argument consumer can advance generator. */
let argumentGate31 = false;
switch (2) {
  case consume((function* () { argumentGate31 = true; yield 1; })()): break;
  case 2: query(argumentGate31 ? "SELECT 31 OFFSET 1" : "SELECT 31"); break; // unanalyzable
}

/* tagged template consumes generator. */
let tagGate32 = false;
switch (2) {
  case consumeTagged`${(function* () { tagGate32 = true; yield 1; })()}`: break;
  case 2: query(tagGate32 ? "SELECT 32 OFFSET 1" : "SELECT 32"); break; // unanalyzable
}

/* tagged template consumes returned generator. */
let tagGate33 = false;
switch (2) {
  case consumeTagged`${(() => (function* () { tagGate33 = true; yield 1; })())()}`: break;
  case 2: query(tagGate33 ? "SELECT 33 OFFSET 1" : "SELECT 33"); break; // unanalyzable
}

/* tagged template consumes parameter default iterator. */
let tagGate34 = false;
switch (2) {
  case consumeTagged`${((iterator = (function* () { tagGate34 = true; yield 1; })()) => iterator)()}`: break;
  case 2: query(tagGate34 ? "SELECT 34 OFFSET 1" : "SELECT 34"); break; // unanalyzable
}

/* plain template coercion leaves iterator inert. */
let tagGate35 = false;
switch (2) {
  case `${(function* () { tagGate35 = true; yield 1; })()}`: break;
  case 2: query(tagGate35 ? "SELECT 35 OFFSET 1" : "SELECT 35"); break; // pass
}

/* tagged callback creation remains inert. */
let tagGate36 = false;
switch (2) {
  case consumeTagged`${function* () { tagGate36 = true; yield 1; }}`: break;
  case 2: query(tagGate36 ? "SELECT 36 OFFSET 1" : "SELECT 36"); break; // pass
}
