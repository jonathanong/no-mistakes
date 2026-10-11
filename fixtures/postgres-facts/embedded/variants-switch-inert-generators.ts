import { query } from "@example/db";
declare function consume(...values: unknown[]): number;
declare function consumeTagged(strings: TemplateStringsArray, value: unknown): number;

/* direct generator creation. */
let gate1 = false;
switch (2) {
  case (function* () { gate1 = true; yield 1; })(): break;
  case 2: query(gate1 ? "SELECT 1 OFFSET 1" : "SELECT 1"); break; // variants: SELECT 1
}

/* TypeScript-wrapped generator creation. */
let gate2 = false;
switch (2) {
  case ((function* () { gate2 = true; }) as () => Generator)(): break;
  case 2: query(gate2 ? "SELECT 2 OFFSET 1" : "SELECT 2"); break; // variants: SELECT 2
}

/* async generator creation. */
let gate3 = false;
switch (2) {
  case (async function* () { gate3 = true; yield 1; })(): break;
  case 2: query(gate3 ? "SELECT 3 OFFSET 1" : "SELECT 3"); break; // variants: SELECT 3
}

/* evaluated generator argument. */
let gate4 = false;
switch (2) {
  case (function* (value) { yield value; })(gate4 = true): break;
  case 2: query(gate4 ? "SELECT 4 OFFSET 1" : "SELECT 4"); break; // opaque
}

/* generator parameter default. */
let gate5 = false;
switch (2) {
  case (function* (value = (gate5 = true)) { yield value; })(): break;
  case 2: query(gate5 ? "SELECT 5 OFFSET 1" : "SELECT 5"); break; // opaque
}

/* generator destructuring default. */
let gate6 = false;
switch (2) {
  case (function* ({ value = (gate6 = true) } = {}) { yield value; })(): break;
  case 2: query(gate6 ? "SELECT 6 OFFSET 1" : "SELECT 6"); break; // opaque
}

/* invoked generator argument closure. */
let gate7 = false;
switch (2) {
  case (function* (value) { yield value; })((() => { gate7 = true; return 0; })()): break;
  case 2: query(gate7 ? "SELECT 7 OFFSET 1" : "SELECT 7"); break; // opaque
}

/* ordinary invoked function. */
let gate8 = false;
switch (2) {
  case (function () { gate8 = true; return 1; })(): break;
  case 2: query(gate8 ? "SELECT 8 OFFSET 1" : "SELECT 8"); break; // opaque
}

/* ordinary invoked arrow. */
let gate9 = false;
switch (2) {
  case (() => { gate9 = true; return 1; })(): break;
  case 2: query(gate9 ? "SELECT 9 OFFSET 1" : "SELECT 9"); break; // opaque
}

/* unused generator in ordinary function. */
let gate10 = false;
switch (2) {
  case (function () { (function* () { gate10 = true; })(); return 1; })(): break;
  case 2: query(gate10 ? "SELECT 10 OFFSET 1" : "SELECT 10"); break; // variants: SELECT 10
}

/* pure generator creation. */
let gate11 = false;
switch (2) {
  case (function* () { yield 1; })(): break;
  case 2: query(gate11 ? "SELECT 11 OFFSET 1" : "SELECT 11"); break; // variants: SELECT 11
}

/* immediate next. */
let gate12 = false;
switch (2) {
  case (function* () { gate12 = true; yield 1; })().next(): break;
  case 2: query(gate12 ? "SELECT 12 OFFSET 1" : "SELECT 12"); break; // opaque
}

/* computed next. */
let gate13 = false;
switch (2) {
  case (function* () { gate13 = true; yield 1; })()["next"](): break;
  case 2: query(gate13 ? "SELECT 13 OFFSET 1" : "SELECT 13"); break; // opaque
}

/* computed receiver key write. */
let gate14 = false;
switch (2) {
  case (function* () { yield 1; })()[(gate14 = true, "next")](): break;
  case 2: query(gate14 ? "SELECT 14 OFFSET 1" : "SELECT 14"); break; // opaque
}

/* optional iterator method. */
let gate15 = false;
switch (2) {
  case (function* () { gate15 = true; yield 1; })().next?.(): break;
  case 2: query(gate15 ? "SELECT 15 OFFSET 1" : "SELECT 15"); break; // opaque
}

/* for-of iteration. */
let gate16 = false;
switch (2) {
  case (() => { for (const value of (function* () { gate16 = true; yield 1; })()) {} return 1; })(): break;
  case 2: query(gate16 ? "SELECT 16 OFFSET 1" : "SELECT 16"); break; // opaque
}

/* array spread consumes iterator. */
let gate17 = false;
switch (2) {
  case [...(function* () { gate17 = true; yield 1; })()]: break;
  case 2: query(gate17 ? "SELECT 17 OFFSET 1" : "SELECT 17"); break; // opaque
}

/* argument spread consumes iterator. */
let gate18 = false;
switch (2) {
  case consume(...(function* () { gate18 = true; yield 1; })()): break;
  case 2: query(gate18 ? "SELECT 18 OFFSET 1" : "SELECT 18"); break; // opaque
}

/* unknown iterator consumer. */
let gate19 = false;
switch (2) {
  case consume((function* () { gate19 = true; yield 1; })()): break;
  case 2: query(gate19 ? "SELECT 19 OFFSET 1" : "SELECT 19"); break; // opaque
}

/* advanced delegated generator. */
let gate20 = false;
switch (2) {
  case (function* () { yield* (function* () { gate20 = true; yield 1; })(); })().next(): break;
  case 2: query(gate20 ? "SELECT 20 OFFSET 1" : "SELECT 20"); break; // opaque
}

/* advanced pure yield. */
let gate21 = false;
switch (2) {
  case (function* () { yield 1; yield; })().next(): break;
  case 2: query(gate21 ? "SELECT 21 OFFSET 1" : "SELECT 21"); break; // variants: SELECT 21
}

/* unused delegated generator. */
let gate22 = false;
switch (2) {
  case (function* () { yield* (function* () { gate22 = true; yield 1; })(); })(): break;
  case 2: query(gate22 ? "SELECT 22 OFFSET 1" : "SELECT 22"); break; // variants: SELECT 22
}

/* iterator returned through arrow. */
let gate23 = false;
switch (2) {
  case (() => (function* () { gate23 = true; yield 1; })())().next(): break;
  case 2: query(gate23 ? "SELECT 23 OFFSET 1" : "SELECT 23"); break; // opaque
}

/* iterator returned through function. */
let gate24 = false;
switch (2) {
  case (function () { return (function* () { gate24 = true; yield 1; })(); })().next(): break;
  case 2: query(gate24 ? "SELECT 24 OFFSET 1" : "SELECT 24"); break; // opaque
}

/* returned iterator passed to unknown consumer. */
let gate25 = false;
switch (2) {
  case consume((() => (function* () { gate25 = true; yield 1; })())()): break;
  case 2: query(gate25 ? "SELECT 25 OFFSET 1" : "SELECT 25"); break; // opaque
}

/* unused generator in ordinary arrow. */
let gate26 = false;
switch (2) {
  case (() => { (function* () { gate26 = true; })(); return 1; })(): break;
  case 2: query(gate26 ? "SELECT 26 OFFSET 1" : "SELECT 26"); break; // variants: SELECT 26
}

/* object spread does not iterate. */
let gate27 = false;
switch (2) {
  case ({ ...(function* () { gate27 = true; yield 1; })() }): break;
  case 2: query(gate27 ? "SELECT 27 OFFSET 1" : "SELECT 27"); break; // variants: SELECT 27
}

/* for-in does not iterate. */
let gate28 = false;
switch (2) {
  case (() => { for (const key in (function* () { gate28 = true; yield 1; })()) {} return 1; })(): break;
  case 2: query(gate28 ? "SELECT 28 OFFSET 1" : "SELECT 28"); break; // variants: SELECT 28
}

/* generator function creation. */
let gate29 = false;
switch (2) {
  case (function* () { gate29 = true; yield 1; }): break;
  case 2: query(gate29 ? "SELECT 29 OFFSET 1" : "SELECT 29"); break; // variants: SELECT 29
}

/* generator callback passed without invocation. */
let gate30 = false;
switch (2) {
  case consume(function* () { gate30 = true; yield 1; }): break;
  case 2: query(gate30 ? "SELECT 30 OFFSET 1" : "SELECT 30"); break; // variants: SELECT 30
}

/* array without spread does not iterate. */
let gate31 = false;
switch (2) {
  case [(function* () { gate31 = true; yield 1; })(), , 1]: break;
  case 2: query(gate31 ? "SELECT 31 OFFSET 1" : "SELECT 31"); break; // variants: SELECT 31
}

/* stored iterator is conservative. */
let gate32 = false;
switch (2) {
  case (() => { const iterator = (function* () { gate32 = true; yield 1; })(); iterator.next(); return 1; })(): break;
  case 2: query(gate32 ? "SELECT 32 OFFSET 1" : "SELECT 32"); break; // opaque
}

/* stored iterator alias is conservative. */
let gate33 = false;
switch (2) {
  case (() => { const iterator = (function* () { gate33 = true; yield 1; })(); const alias = iterator; alias.next(); return 1; })(): break;
  case 2: query(gate33 ? "SELECT 33 OFFSET 1" : "SELECT 33"); break; // opaque
}

/* stored unknown consumer is conservative. */
let gate34 = false;
switch (2) {
  case (() => { const iterator = (function* () { gate34 = true; yield 1; })(); consume(iterator); return 1; })(): break;
  case 2: query(gate34 ? "SELECT 34 OFFSET 1" : "SELECT 34"); break; // opaque
}

/* stored unused iterator retains fallback. */
let gate35 = false;
switch (2) {
  case (() => { const iterator = (function* () { gate35 = true; yield 1; })(); return 1; })(): break;
  case 2: query(gate35 ? "SELECT 35 OFFSET 1" : "SELECT 35"); break; // opaque
}

/* Private member invocation also keeps unknown consumption conservative. */
class PrivateConsumer {
  #resume() {}
  run() {
    let privateGate = false;
    switch (2) {
      case (function* () { privateGate = true; yield 1; })().#resume(): break;
      case 2: query(privateGate ? "SELECT 36 OFFSET 1" : "SELECT 36"); break; // opaque
    }
  }
}

/* Legacy literal and immutable SQL keep their public fact fields. */
let stableEffect = false;
const stable = "SELECT 37";
switch (2) {
  case (function* () { stableEffect = true; })(): break;
  case 2:
    query("SELECT 38"); // inline: SELECT 38
    query(stable); // unchanged: SELECT 37
    break;
}

/* ordinary arrow parameter iterator default. */
let parameterGate39 = false;
switch (2) {
  case ((iterator = (function* () { parameterGate39 = true; yield 1; })()) => { iterator.next(); return 1; })(): break;
  case 2: query(parameterGate39 ? "SELECT 39 OFFSET 1" : "SELECT 39"); break; // opaque
}

/* ordinary function parameter iterator default. */
let parameterGate40 = false;
switch (2) {
  case (function (iterator = (function* () { parameterGate40 = true; yield 1; })()) { iterator.next(); return 1; })(): break;
  case 2: query(parameterGate40 ? "SELECT 40 OFFSET 1" : "SELECT 40"); break; // opaque
}

/* wrapped arrow parameter iterator default. */
let parameterGate41 = false;
switch (2) {
  case (((iterator = (function* () { parameterGate41 = true; yield 1; })()) => { iterator.next(); return 1; }) as () => number)(): break;
  case 2: query(parameterGate41 ? "SELECT 41 OFFSET 1" : "SELECT 41"); break; // opaque
}

/* destructured parameter unknown iterator consumer. */
let parameterGate42 = false;
switch (2) {
  case (({ iterator = (function* () { parameterGate42 = true; yield 1; })() } = {}) => { consume(iterator); return 1; })(): break;
  case 2: query(parameterGate42 ? "SELECT 42 OFFSET 1" : "SELECT 42"); break; // opaque
}

/* nested function parameter unknown iterator consumer. */
let parameterGate43 = false;
switch (2) {
  case (function ({ nested: { iterator = (function* () { parameterGate43 = true; yield 1; })() } = {} } = {}) { consume(iterator); return 1; })(): break;
  case 2: query(parameterGate43 ? "SELECT 43 OFFSET 1" : "SELECT 43"); break; // opaque
}

/* array binding default iterator. */
let bindingGate44 = false;
switch (2) {
  case (() => { const [iterator = (function* () { bindingGate44 = true; yield 1; })()] = []; iterator.next(); return 1; })(): break;
  case 2: query(bindingGate44 ? "SELECT 44 OFFSET 1" : "SELECT 44"); break; // opaque
}

/* object binding default iterator. */
let bindingGate45 = false;
switch (2) {
  case (() => { const { iterator = (function* () { bindingGate45 = true; yield 1; })() } = {}; consume(iterator); return 1; })(): break;
  case 2: query(bindingGate45 ? "SELECT 45 OFFSET 1" : "SELECT 45"); break; // opaque
}

/* for-of binding default iterator. */
let bindingGate46 = false;
switch (2) {
  case (() => { for (const [iterator = (function* () { bindingGate46 = true; yield 1; })()] of [[]]) { iterator.next(); } return 1; })(): break;
  case 2: query(bindingGate46 ? "SELECT 46 OFFSET 1" : "SELECT 46"); break; // opaque
}

/* catch binding default iterator. */
let bindingGate47 = false;
switch (2) {
  case (() => { try { throw {}; } catch ({ iterator = (function* () { bindingGate47 = true; yield 1; })() }) { iterator.next(); } return 1; })(): break;
  case 2: query(bindingGate47 ? "SELECT 47 OFFSET 1" : "SELECT 47"); break; // opaque
}

/* tagged template consumes generator. */
let tagGate48 = false;
switch (2) {
  case consumeTagged`${(function* () { tagGate48 = true; yield 1; })()}`: break;
  case 2: query(tagGate48 ? "SELECT 48 OFFSET 1" : "SELECT 48"); break; // opaque
}

/* tagged template consumes returned generator. */
let tagGate49 = false;
switch (2) {
  case consumeTagged`${(() => (function* () { tagGate49 = true; yield 1; })())()}`: break;
  case 2: query(tagGate49 ? "SELECT 49 OFFSET 1" : "SELECT 49"); break; // opaque
}

/* tagged template consumes parameter default iterator. */
let tagGate50 = false;
switch (2) {
  case consumeTagged`${((iterator = (function* () { tagGate50 = true; yield 1; })()) => iterator)()}`: break;
  case 2: query(tagGate50 ? "SELECT 50 OFFSET 1" : "SELECT 50"); break; // opaque
}

/* plain template coercion leaves iterator inert. */
let tagGate51 = false;
switch (2) {
  case `${(function* () { tagGate51 = true; yield 1; })()}`: break;
  case 2: query(tagGate51 ? "SELECT 51 OFFSET 1" : "SELECT 51"); break; // variants: SELECT 51
}

/* tagged callback creation remains inert. */
let tagGate52 = false;
switch (2) {
  case consumeTagged`${function* () { tagGate52 = true; yield 1; }}`: break;
  case 2: query(tagGate52 ? "SELECT 52 OFFSET 1" : "SELECT 52"); break; // variants: SELECT 52
}
