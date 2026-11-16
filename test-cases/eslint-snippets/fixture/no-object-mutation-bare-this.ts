new Readable({ read() { this.push(null); } });
export const stream = {
  read() {
    this.push(null);
    this.pop();
    this.shift();
    this.unshift(null);
    this.splice(0, 1);
    this.sort();
    this.reverse();
    this.fill(null);
    this.copyWithin(0, 1);
    (() => this.push(null))();
  },
};
export function read() {
  this.push(null);
  this.pop();
}
