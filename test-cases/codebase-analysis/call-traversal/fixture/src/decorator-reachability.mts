import { classDecorator, memberDecorator } from "./decorator-target.mts";
import * as dynamicDecorators from "./dynamic-decorator-target.mts";

function outer() {
  @classDecorator
  class Service {
    @memberDecorator
    field = 1;

    @(dynamicDecorators[method])
    method() {}
  }
}

outer();
