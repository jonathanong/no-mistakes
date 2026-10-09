"use strict";
// Program strictness reaches ordinary functions and functions nested in arrows.
function strictFunction(value) {}
const strictArrow = () => {
  function strictNested(value) {}
};
