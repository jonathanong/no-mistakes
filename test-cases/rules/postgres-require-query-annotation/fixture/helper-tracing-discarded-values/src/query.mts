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
