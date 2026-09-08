function first() {
  second();
}

function second() {
  third();
}

function third() {}

function outer() {
  function outerOnly() {}

  function inner() {
    outerOnly();
  }

  inner();
}

first();
outer();
