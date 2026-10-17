import { parameterCall, bodyCall, nestedLoopCall, nestedBodyCall, bodyAfterCall, arrowParameterCall, arrowBodyCall, objectKeyCall, objectBodyCall, decoratorCall, classKeyCall, classBodyCall, fieldCall, fieldArrowCall, staticCall, staticBlockCall, loopAfterCall, outsideFieldCall, outsideArrowBodyCall, outsideStaticCall, outsideStaticBlockCall, moduleAfterCall, tagBody, constructBody, objectNamedBodyCall, classNamedBodyCall, overloadBodyCall, classOverloadBodyCall, defaultBodyCall, defaultLoopCall, outsideKeyCall, outsideUnknownBodyCall } from '@vendor/valkey';
for (const value of [0, 1]) {
  function declared(param = parameterCall()) {
    bodyCall(param);
    for (const nested of [0, 1]) {
      nestedLoopCall(nested);
      function nestedDeclaration() { nestedBodyCall(); }
    }
    bodyAfterCall();
    tagBody`value`;
    new constructBody();
  }
  function overloaded(param: number): void;
  function overloaded(param: string): void;
  function overloaded(param: unknown) { overloadBodyCall(param); }
  const createdArrow = (param = arrowParameterCall()) => arrowBodyCall(param);
  const object = { [objectKeyCall()]() { objectBodyCall(); }, named() { objectNamedBodyCall(); } };
  @decoratorCall()
  class Created {
    [classKeyCall()]() { classBodyCall(); }
    named() { classNamedBodyCall(); }
    overloaded(param: number): void;
    overloaded(param: unknown) { classOverloadBodyCall(param); }
    field = fieldCall();
    arrow = () => fieldArrowCall();
    static field = staticCall();
    static { staticBlockCall(); }
  }
  loopAfterCall(value);
}
class Outside {
  [outsideKeyCall()]() { outsideUnknownBodyCall(); }
  field = outsideFieldCall();
  arrow = () => outsideArrowBodyCall();
  static field = outsideStaticCall();
  static { outsideStaticBlockCall(); }
}
moduleAfterCall();

export default function exported(value: number): void;
export default function exported(value: string): void;
export default function exported(value: unknown) {
  for (const item of [0, 1]) {
    const inDefault = () => defaultBodyCall(value);
    defaultLoopCall(item);
  }
}
