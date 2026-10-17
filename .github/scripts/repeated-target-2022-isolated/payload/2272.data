import {
  emptyReceiver, conditionalReceiver, earlyReceiver,
  resetReceiver, escapedReceiver, shadowReceiver, unknownReceiver,
  returnedReceiver, passedReceiver, recursiveReceiver, selfShadowReceiver,
  variableRecursiveReceiver,
  preuseReceiver,
  nestedReturnedReceiver, nestedPassedReceiver, parameterShadowReceiver,
  nestedShadowReceiver,
} from "@vendor/valkey";

function replacement() {}
declare const flag: boolean;
declare const unknownValue: unknown;
// Each case removes one part of the closed assignment/call proof.

let emptyDeferred;
for (const value of []) {
  emptyDeferred = function emptyIteration() {
    const saved = emptyReceiver.get;
    (function takeEmpty(fn) { fn(value); })(saved);
    emptyReceiver.get = replacement;
  };
}
emptyDeferred();

let conditionalDeferred;
for (const value of [0, 1]) {
  if (flag) conditionalDeferred = function conditionalIteration() {
    const saved = conditionalReceiver.get;
    (function takeConditional(fn) { fn(value); })(saved);
    conditionalReceiver.get = replacement;
  };
}
conditionalDeferred();

let earlyDeferred;
for (const value of [0, 1]) {
  break;
  earlyDeferred = function earlyExit() {
    const saved = earlyReceiver.get;
    (function takeEarly(fn) { fn(value); })(saved);
    earlyReceiver.get = replacement;
  };
}
earlyDeferred();

let resetDeferred;
for (const value of [0, 1]) {
  resetDeferred = function resetAfter() {
    const saved = resetReceiver.get;
    (function takeReset(fn) { fn(value); })(saved);
    resetReceiver.get = replacement;
  };
}
resetDeferred = unknownValue;
resetDeferred();

let escapedDeferred;
for (const value of [0, 1]) {
  escapedDeferred = function escapedValue() {
    const saved = escapedReceiver.get;
    (function takeEscaped(fn) { fn(value); })(saved);
    escapedReceiver.get = replacement;
  };
}
const retained = escapedDeferred;
escapedDeferred();
void retained;

let unknownDeferred = unknownValue;
for (const value of [0, 1]) {
  unknownDeferred = function unknownInitializer() {
    const saved = unknownReceiver.get;
    (function takeUnknown(fn) { fn(value); })(saved);
    unknownReceiver.get = replacement;
  };
}
unknownDeferred();

let shadowDeferred;
for (const value of [0, 1]) {
  shadowDeferred = function outerShadow() {
    const saved = shadowReceiver.get;
    (function takeShadow(fn) { fn(value); })(saved);
    shadowReceiver.get = replacement;
  };
}
{
  // This lexical shadow is unrelated to the mutable module binding.
  const shadowDeferred = () => {};
  shadowDeferred();
}
shadowDeferred();

let returnedDeferred;
for (const value of [0, 1]) {
  returnedDeferred = function returnedSelf() {
    const saved = returnedReceiver.get;
    (function takeReturned(fn) { fn(value); })(saved);
    returnedReceiver.get = replacement;
    return returnedSelf;
  };
}
returnedDeferred();

function consume(value) { return value; }
let passedDeferred;
for (const value of [0, 1]) {
  passedDeferred = function passedSelf() {
    const saved = passedReceiver.get;
    (function takePassed(fn) { fn(value); })(saved);
    passedReceiver.get = replacement;
    consume(passedSelf);
  };
}
passedDeferred();

let recursiveDeferred;
for (const value of [0, 1]) {
  recursiveDeferred = function recursiveSelf() {
    const saved = recursiveReceiver.get;
    (function takeRecursive(fn) { fn(value); })(saved);
    recursiveReceiver.get = replacement;
    recursiveSelf();
  };
}
recursiveDeferred();

let variableRecursiveDeferred;
for (const value of [0, 1]) {
  variableRecursiveDeferred = function recursiveVariable() {
    const saved = variableRecursiveReceiver.get;
    (function takeVariableRecursive(fn) { fn(value); })(saved);
    variableRecursiveReceiver.get = replacement;
    variableRecursiveDeferred();
  };
}
variableRecursiveDeferred();

let selfShadowDeferred;
for (const value of [0, 1]) {
  selfShadowDeferred = function selfShadow() {
    {
      // A same-spelled local value is not a use of the function's self-binding.
      const selfShadow = 1;
      void selfShadow;
    }
    const saved = selfShadowReceiver.get;
    (function takeSelfShadow(fn) { fn(value); })(saved);
    selfShadowReceiver.get = replacement;
  };
}
selfShadowDeferred();

// Program predeclaration exposes this escape even before the later assignment.
function leakBefore() { return preuseDeferred; }
let preuseDeferred;
for (const value of [0, 1]) {
  preuseDeferred = function preused() {
    const saved = preuseReceiver.get;
    (function takePreused(fn) { fn(value); })(saved);
    preuseReceiver.get = replacement;
  };
}
preuseDeferred();
void leakBefore;

let nestedReturnedDeferred;
for (const value of [0, 1]) {
  nestedReturnedDeferred = function nestedReturned() {
    const saved = nestedReturnedReceiver.get;
    (function takeNestedReturned(fn) { fn(value); })(saved);
    nestedReturnedReceiver.get = replacement;
    return () => nestedReturned;
  };
}
nestedReturnedDeferred();

let nestedPassedDeferred;
for (const value of [0, 1]) {
  nestedPassedDeferred = function nestedPassed() {
    const saved = nestedPassedReceiver.get;
    (function takeNestedPassed(fn) { fn(value); })(saved);
    nestedPassedReceiver.get = replacement;
    consume(() => nestedPassed);
  };
}
nestedPassedDeferred();

let parameterShadowDeferred;
for (const value of [0, 1]) {
  parameterShadowDeferred = function parameterShadow(parameterShadow) {
    const saved = parameterShadowReceiver.get;
    (function takeParameterShadow(fn) { fn(value); })(saved);
    parameterShadowReceiver.get = replacement;
    return parameterShadow;
  };
}
parameterShadowDeferred();

let nestedShadowDeferred;
for (const value of [0, 1]) {
  nestedShadowDeferred = function nestedShadow() {
    const saved = nestedShadowReceiver.get;
    (function takeNestedShadow(fn) { fn(value); })(saved);
    nestedShadowReceiver.get = replacement;
    return () => {
      // The closure's local shadows the named function-expression binding.
      const nestedShadow = 1;
      return nestedShadow;
    };
  };
}
nestedShadowDeferred();
