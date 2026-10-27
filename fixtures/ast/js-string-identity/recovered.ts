// Keep static names and computed members on the same legacy identity encoding.
export { value as "ordinary", value as "�d800" };
const object = { "\uD800": 1, 42: 2, "\u{D800}\u{DC00}": 3, [`\u{D800}\u{DC00}`]: 4 };
object["\u{D800}\u{DC00}"];
object[`\u{D800}\u{DC00}`];
object.plain;
