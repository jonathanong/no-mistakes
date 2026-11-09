import { owner } from './owner.mjs';

export function untypedNumber(numberLike) { numberLike.toFixed(1); }
export function untypedJoin(joinable) { joinable.join(','); }
export function untypedHas(hasValue) { hasValue.has(1); }
export function untypedCustom(unknownValue) { unknownValue.custom(); }
export function typedWrong(textValue: string) { textValue.toFixed(1); }
export function customSink(sinkValue: { toFixed(digits: number): string }) { sinkValue.toFixed(1); }
export function unknownNumber(input: unknown) { input.toFixed(1); }
export function anyNumber(inputAny: any) { inputAny.toFixed(1); }
export function localUnknown() { const computed: unknown = undefined; computed.toFixed(1); }
export function resultUnknown() { Math.random().toFixed(1); }
declare function unknownFactory(): unknown;
export function resultUnknownFactory() { unknownFactory().toFixed(1); }
export function wrongLiteral() { const text = 'x'; text.toFixed(1); }
export function unknownFind(candidate) { candidate.find(owner); }
type Sink = { join(separator: string): string };
export function castedBinding() { const casted = ({} as Sink); casted.join(','); }
export function directCasted(base: unknown) { (base as Sink).join(','); }
export function directAssertion(base: unknown) { (<Sink>base).join(','); }
export function directSatisfies() { ({ join: (_: string) => '' } satisfies Sink).join(','); }
export function unknownCast(base: unknown) { (base as unknown).join(','); }
export function anyCast(base: any) { (base as any).join(','); }
export function unknownCastBinding() { const castedUnknown = ({} as unknown); castedUnknown.join(','); }
export function arrayFindOwner() {
  const ownerRows = [1, 2];
  ownerRows.join(',');
  ownerRows.find(owner);
}
export function arrayFindNoOwner() {
  const plainRows = [1, 2];
  plainRows.join(',');
  plainRows.find(value => value > 0);
}
export function setHas() {
  const membership = new Set([1]);
  membership.has(1);
}
