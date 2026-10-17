{
  const { open } = other.panel;
  open();
}
{
  const { open: show } = other.panel;
}
{
  const { panel: { open } } = other;
}
{
  const { panel: { open: show } } = other;
}
{
  const { close, open, ...rest } = other.panel;
}
{
  ({ open } = other.panel);
}
{
  const { open: { visible } } = other.panel;
}
{
  const { entry } = records;
}
{
  const { entry: name } = records;
}
// Named roots are spellings, not bindings. The parameter does not hide the chain.
function f(records) {
  return records.entry;
}
{
  const { entry } = other.records;
}
{
  const { open } = panel;
}
{
  // Computed keys are unproven, including a literal computed key.
  const { [key]: open } = other.panel;
  const { ["open"]: shown } = other.panel;
}
{
  const { open: fallback = other.panel } = {};
}
{
  const { open } = other[name].panel;
}
{
  const { entry } = obj[name];
}
{
  const { toggle } = other.drawer;
}
{
  let { open: late } = other.panel;
}
{
  const { "open": quoted } = other.panel;
}
{
  const { 1: slot } = matrix;
  const { "1": slotName } = matrix;
}
{
  const { open = false } = other.panel;
}
{
  const { panel: { open: nested = false } = {} } = other;
}
function g({ open } = other.panel) {
  return open;
}
{
  const use = ({ open } = other.panel) => open;
}
function h({ open }) {
  return open;
}
{
  for (const { open } of panels) open;
}
{
  const { open: { panel: { open: inner } } } = other.panel;
}
{
  // no-mistakes-disable-next-line forbidden-member-chains
  const { open: hidden } = other.panel;
}
{
  const { open } = other?.panel;
}
{
  const { open } = (other.panel);
}
{
  const { open } = this.panel;
}
class Child extends Base {
  method() {
    const { open } = super.panel;
    return open;
  }
}
{
  const { 1n: wide } = matrix;
}
const methods = {
  take({ open } = other.panel) {
    return open;
  },
};
function plain(open = other.panel) {
  return open;
}
// 33 members on the receiver are unproven, same as a direct chain.
{
  const { open } =
    root.p1.p2.p3.p4.p5.p6.p7.p8.p9.p10.p11.p12.p13.p14.p15.p16.p17.p18.p19.p20.p21.p22.p23.p24.p25.p26.p27.p28.p29.p30.p31.panel.open;
}
