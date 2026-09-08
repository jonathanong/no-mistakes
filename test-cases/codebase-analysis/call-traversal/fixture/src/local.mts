function first() {
  second();
}

function second() {
  third();
}

function third() {}

first();
