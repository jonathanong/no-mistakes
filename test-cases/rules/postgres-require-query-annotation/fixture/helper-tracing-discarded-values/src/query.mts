import sql from "sql-template-strings";
import { write } from "@app/db";

const initializerSequenceStatement = sql`/* initializer sequence callback */ SELECT 1`;
const initializerSequenceCallback = () => unknownMutation(initializerSequenceStatement);
const sequenceValue = unknownConsumer((initializerSequenceCallback, 0));
write(initializerSequenceStatement); // known:initializer-sequence-does-not-escape

const initializerVoidStatement = sql`/* initializer void callback */ SELECT 1`;
const initializerVoidCallback = () => unknownMutation(initializerVoidStatement);
const voidValue = unknownConsumer(void initializerVoidCallback);
write(initializerVoidStatement); // known:initializer-void-does-not-escape

const sequenceStatement = sql`/* sequence callback */ SELECT 1`;
const sequenceCallback = () => unknownMutation(sequenceStatement);
unknownConsumer((sequenceCallback, 0));
write(sequenceStatement); // known:sequence-does-not-escape-callback

const voidStatement = sql`/* void callback */ SELECT 1`;
const voidCallback = () => unknownMutation(voidStatement);
unknownConsumer(void voidCallback);
write(voidStatement); // known:void-does-not-escape-callback

const sequenceEffectStatement = sql`/* sequence side effect */ SELECT 1`;
unknownConsumer((unknownMutation(sequenceEffectStatement), 0));
write(sequenceEffectStatement); // unanalyzable:sequence-keeps-operand-effects

const voidEffectStatement = sql`/* void side effect */ SELECT 1`;
unknownConsumer(void unknownMutation(voidEffectStatement));
write(voidEffectStatement); // unanalyzable:void-keeps-operand-effects

const survivingCallbackStatement = sql`/* surviving callback */ SELECT 1`;
const survivingCallback = () => unknownMutation(survivingCallbackStatement);
unknownConsumer((0, survivingCallback));
write(survivingCallbackStatement); // unanalyzable:sequence-exposes-final-callback

const unknownFinalStatement = sql`/* unknown final argument */ SELECT 1`;
const discardedCallback = () => unknownMutation(unknownFinalStatement);
unknownConsumer((discardedCallback, unknownFinalArgument));
write(unknownFinalStatement); // unanalyzable:sequence-has-unknown-final-argument

// Empty objects and arrays are references, rather than primitive projections.
const objectFinalStatement = sql`/* object final argument */ SELECT 1`;
const objectDiscardedCallback = () => unknownMutation(objectFinalStatement);
unknownConsumer((objectDiscardedCallback, {}));
write(objectFinalStatement); // unanalyzable:sequence-has-object-final-argument

const arrayFinalStatement = sql`/* array final argument */ SELECT 1`;
const arrayDiscardedCallback = () => unknownMutation(arrayFinalStatement);
unknownConsumer((arrayDiscardedCallback, []));
write(arrayFinalStatement); // unanalyzable:sequence-has-array-final-argument

const unaryPlusStatement = sql`/* unary plus coercion */ SELECT 1`;
unknownConsumer(+unaryPlusStatement);
write(unaryPlusStatement); // unanalyzable:unary-plus-may-run-coercion-hooks

const unaryNegationStatement = sql`/* unary negation coercion */ SELECT 1`;
unknownConsumer(-unaryNegationStatement);
write(unaryNegationStatement); // unanalyzable:unary-negation-may-run-coercion-hooks

const bitwiseNotStatement = sql`/* bitwise not coercion */ SELECT 1`;
unknownConsumer(~bitwiseNotStatement);
write(bitwiseNotStatement); // unanalyzable:bitwise-not-may-run-coercion-hooks

const typeofStatement = sql`/* typeof is noncoercive */ SELECT 1`;
unknownConsumer(typeof typeofStatement);
write(typeofStatement); // known:typeof-does-not-run-coercion-hooks

const logicalNotStatement = sql`/* logical not is noncoercive */ SELECT 1`;
unknownConsumer(!logicalNotStatement);
write(logicalNotStatement); // known:logical-not-does-not-run-coercion-hooks

const getterStatement = sql`/* getter before consumer */ SELECT 1`;
unknownConsumer(void proxy.trigger);
write(getterStatement); // unanalyzable:void-member-can-run-getter

function discardedArgumentSlot(statement) {
  unknownConsumer(void arguments[0]);
  return statement;
}
write(discardedArgumentSlot(sql`/* discarded argument slot */ SELECT 1`)); // known:argument-slot-read-is-data

function discardedArgumentLength(statement) {
  unknownConsumer(void arguments.length);
  return statement;
}
write(discardedArgumentLength(sql`/* discarded argument length */ SELECT 1`)); // known:argument-length-read-is-data

function discardedArgumentAlias(statement) {
  const slots = arguments;
  unknownConsumer(void slots[0]);
  return statement;
}
write(discardedArgumentAlias(sql`/* discarded argument alias */ SELECT 1`)); // known:argument-alias-read-is-data

function discardedArgumentProperty(statement) {
  unknownConsumer(void arguments[0].trigger);
  return statement;
}
write(discardedArgumentProperty(sql`/* argument property getter */ SELECT 1`)); // unanalyzable:void-argument-property-can-run-getter

function discardedNamedArgument(statement) {
  unknownConsumer(void arguments.trigger);
  return statement;
}
write(discardedNamedArgument(sql`/* named argument accessor */ SELECT 1`)); // unanalyzable:named-argument-property-can-run-getter
