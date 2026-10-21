// Retain private closures so successive branches have a large preserved prefix.
declare const flag: boolean;
function capture(value: string) {
  return () => value;
}
const keep00 = capture("/* sparse00 */ SELECT 0");
const keep01 = capture("/* sparse01 */ SELECT 1");
const keep02 = capture("/* sparse02 */ SELECT 2");
const keep03 = capture("/* sparse03 */ SELECT 3");
const keep04 = capture("/* sparse04 */ SELECT 4");
const keep05 = capture("/* sparse05 */ SELECT 5");
const keep06 = capture("/* sparse06 */ SELECT 6");
const keep07 = capture("/* sparse07 */ SELECT 7");
const keep08 = capture("/* sparse08 */ SELECT 8");
const keep09 = capture("/* sparse09 */ SELECT 9");
const keep10 = capture("/* sparse10 */ SELECT 10");
const keep11 = capture("/* sparse11 */ SELECT 11");
const keep12 = capture("/* sparse12 */ SELECT 12");
const keep13 = capture("/* sparse13 */ SELECT 13");
const keep14 = capture("/* sparse14 */ SELECT 14");
const keep15 = capture("/* sparse15 */ SELECT 15");
const keep16 = capture("/* sparse16 */ SELECT 16");
const keep17 = capture("/* sparse17 */ SELECT 17");
const keep18 = capture("/* sparse18 */ SELECT 18");
const keep19 = capture("/* sparse19 */ SELECT 19");
const keep20 = capture("/* sparse20 */ SELECT 20");
const keep21 = capture("/* sparse21 */ SELECT 21");
const keep22 = capture("/* sparse22 */ SELECT 22");
const keep23 = capture("/* sparse23 */ SELECT 23");

flag ? keep00 : keep00;
flag ? keep01 : keep01;
flag ? keep02 : keep02;
flag ? keep03 : keep03;
flag ? keep04 : keep04;
flag ? keep05 : keep05;
flag ? keep06 : keep06;
flag ? keep07 : keep07;
flag ? keep08 : keep08;
flag ? keep09 : keep09;
flag ? keep10 : keep10;
flag ? keep11 : keep11;
flag ? keep12 : keep12;
flag ? keep13 : keep13;
flag ? keep14 : keep14;
flag ? keep15 : keep15;
flag ? keep16 : keep16;
flag ? keep17 : keep17;
flag ? keep18 : keep18;
flag ? keep19 : keep19;
flag ? keep20 : keep20;
flag ? keep21 : keep21;
flag ? keep22 : keep22;
flag ? keep23 : keep23;

export const retained = keep00();
