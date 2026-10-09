let direct, updated, arrayTarget, objectTarget, restTarget;
let asTarget, satisfiesTarget, nonNullTarget, assertedTarget;
let receiver, key, fallback, input;
direct = input;
updated++;
[arrayTarget = fallback, ...restTarget] = input;
({ item: objectTarget = fallback } = input);
// Member receivers and computed keys are reads, not binding assignments.
receiver.field = input;
receiver[key] = input;
(asTarget as any) = input;
(satisfiesTarget satisfies any) = input;
nonNullTarget! = input;
(<any>assertedTarget) = input;
(receiver.field as any) = input;
[direct, direct] = input;
class PrivateTarget {
  #field;
  change(input) { this.#field = input; }
}
