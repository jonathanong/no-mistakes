const JSON = {
  parse(_text: string, reviver: () => number) { return reviver(); },
  stringify(_value: unknown, replacer: () => number) { return replacer(); },
};
const callback = () => 1;
JSON.parse('{"value":1}', callback);
JSON.stringify({ value: 1 }, callback);
