import { write } from "@app/db";
write(1 + 2); // numeric-add
write(9007199254740990 + 2); // numeric-overflow
write(3 - 2); // numeric-subtract-opaque
write(3 * 2); // numeric-multiply-opaque
write(3 / 2); // numeric-divide-opaque
write(3 === 3); // numeric-strict-equal
write(3 === 2); // numeric-strict-unequal
write(3 == 3); // numeric-equal
write(3 != 2); // numeric-unequal
write(3 !== 3); // numeric-strict-not-equal
write(3 > 2); // numeric-greater
write(2 > 3); // numeric-not-greater
write(2 < 3); // numeric-less
write(3 < 2); // numeric-not-less
write(3 >= 3); // numeric-greater-equal
write(2 >= 3); // numeric-not-greater-equal
write(3 <= 3); // numeric-less-equal
write(3 <= 2); // numeric-not-less-equal
write("SELECT " + "1"); // text-add
write("SELECT " + 1); // mixed-add
write("a" === "a"); // text-comparison-opaque
write({ value: 1 } + 2); // record-coercion
write(1 + { value: 2 }); // reversed-record-coercion
write({ value: 1 } === { value: 1 }); // strict-record-comparison
write([1] + 2); // array-coercion
write(+0); // unary-zero
write(-0); // unary-negative-zero
write(1.25); // fractional-primitive
write(9007199254740991); // unsafe-integer-primitive
