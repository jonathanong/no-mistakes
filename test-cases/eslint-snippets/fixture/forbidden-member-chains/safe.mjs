const label = 'records.entry';
const template = `other.panel.open`;
// other.panel.open stays a comment
/*
 records.entry
*/
const text = 'no-mistakes-disable-file forbidden-member-chains';
panel.open;
records.entries;
other.records.entry;
other.panel.close;
other.panel;
// Computed keys that are not literals are unproven and must not match.
other.panel[name];
other[name].open;
other.panel['open' + ''];
other.panel[name()];
other.panel[/open/];
other.panel[true];
other.panel[-1];
other.panel[`open${''}`];
matrix[1n];
entry;
records;
function open(panel) {
  return panel;
}
other.drawer.toggleExtra;
other.drawer;
drawer.toggle;
