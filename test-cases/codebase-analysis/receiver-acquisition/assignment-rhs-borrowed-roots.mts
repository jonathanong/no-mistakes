import { primary, replica } from '@vendor/valkey';

function replacement() {}
function consume(value) { value.client.get('borrowed-root'); }

let mutable = { client: primary };
const immutable = { client: replica };
const box = { selected: immutable };

// Only the selected member acquires a receiver; the sibling is a decoy.
const objectMember = ({ selected: mutable, ignored: immutable }).selected;
const arrayMember = [immutable, mutable][1];
const nestedMember = ({ outer: { selected: mutable, ignored: immutable } }).outer.selected;
const dottedMember = mutable.client;
const missingMember = ({ ignored: mutable }).missing;

// Unprojected containers acquire all fields, unlike a projected literal.
const wholeObject = { mutable, immutable };
const wholeArray = [mutable, immutable];
const { selected: defaultMember = mutable } = {};
const [defaultArray = mutable] = [];

box.selected = objectMember;
box.selected.client.get = replacement;
box.selected = arrayMember;
box.selected.client.get = replacement;
box.selected = nestedMember;
box.selected.client.get = replacement;
box.selected = defaultMember;
box.selected.client.get = replacement;
box.selected = defaultArray;
box.selected.client.get = replacement;

// The inner spelling must resolve to its own immutable binding.
function shadowed() {
  const mutable = immutable;
  const selected = ({ selected: mutable }).selected;
  consume(selected);
}

async function awaited() {
  const selected = await ({ selected: mutable }).selected;
  box.selected = selected;
  box.selected.client.get = replacement;
  consume(selected);
}

consume(objectMember);
consume(arrayMember);
consume(nestedMember);
consume(defaultMember);
consume(defaultArray);
consume(wholeObject.mutable);
consume(wholeArray[0]);
dottedMember.get('dotted-root');
void missingMember;
void shadowed;
void awaited;
