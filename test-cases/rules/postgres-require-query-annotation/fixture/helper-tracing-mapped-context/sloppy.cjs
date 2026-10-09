// Only sloppy ordinary functions with simple parameters have mapped arguments.
function simple(value) {}
function duplicate(value, value) {}
function defaults(value = "default") {}
function destructured({ value }) {}
function rest(...values) {}
const arrow = (value) => value;
const sloppyArrow = () => {
  function nestedSloppy(value) {}
};
function ownStrict(value) {
  "use strict";
  function nestedStrict(value) {}
  const strictArrow = () => {
    function nestedStrictArrow(value) {}
  };
}
const ownStrictArrow = () => {
  "use strict";
  function nestedOwnStrictArrow(value) {}
};
class StrictClass extends function heritage(value) {
  return Object;
} {
  method(value) {
    function nestedClass(value) {}
  }
}
