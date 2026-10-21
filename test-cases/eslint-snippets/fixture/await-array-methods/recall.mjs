const items = [3, 1, 2];
const alias = items;
await items[`sort`]();
await (items?.sort)();
await alias.sort();
await items.sort.call(items);
await (0, items.sort)();
await items.sort.apply(items, []);

// A private member is not Array.prototype.sort, so this stays unreported.
class Sorter {
  #sort() {
    return 0;
  }
  async run(items) {
    await items.#sort();
  }
}
