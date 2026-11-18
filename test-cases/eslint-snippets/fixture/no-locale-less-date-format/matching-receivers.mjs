(1234).toLocaleString();
Number(1234).toLocaleString(undefined);
const formatter = {
  toLocaleString(locale) {
    return locale;
  },
  toLocaleDateString(locale) {
    return locale;
  },
  toLocaleTimeString(locale) {
    return locale;
  },
};
formatter.toLocaleString();
formatter.toLocaleDateString(undefined);
formatter.toLocaleTimeString("en-US");
export function formatUnknown(receiver) {
  return receiver.toLocaleString();
}
export class PrivateFormatter {
  #toLocaleString() {
    return "formatted";
  }
  format() {
    return this.#toLocaleString();
  }
}
