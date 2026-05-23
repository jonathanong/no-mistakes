function transforms(value: string) {
  return original(value.trim());
}

function addsWork(value: string) {
  audit(value);
  return original(value);
}

const changesArguments = (value: string) => original(value, "extra");
const literal = () => 42;

function original(value: string, extra?: string) {
  return extra ? value + extra : value;
}

function audit(_value: string) {}
