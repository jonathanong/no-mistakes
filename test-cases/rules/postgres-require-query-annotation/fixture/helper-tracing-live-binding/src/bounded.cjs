// Each independent local write has one target despite earlier call frames.
function make(parameter) {
  const local = "SELECT 1";
  return () => parameter;
}
const call0 = make("/* annotation */ SELECT 1");
const call1 = make("/* annotation */ SELECT 1");
const call2 = make("/* annotation */ SELECT 1");
const call3 = make("/* annotation */ SELECT 1");
const call4 = make("/* annotation */ SELECT 1");
const call5 = make("/* annotation */ SELECT 1");
const call6 = make("/* annotation */ SELECT 1");
const call7 = make("/* annotation */ SELECT 1");
const call8 = make("/* annotation */ SELECT 1");
const call9 = make("/* annotation */ SELECT 1");
const call10 = make("/* annotation */ SELECT 1");
const call11 = make("/* annotation */ SELECT 1");
const call12 = make("/* annotation */ SELECT 1");
const call13 = make("/* annotation */ SELECT 1");
const call14 = make("/* annotation */ SELECT 1");
const call15 = make("/* annotation */ SELECT 1");
const call16 = make("/* annotation */ SELECT 1");
const call17 = make("/* annotation */ SELECT 1");
const call18 = make("/* annotation */ SELECT 1");
const call19 = make("/* annotation */ SELECT 1");
const call20 = make("/* annotation */ SELECT 1");
const call21 = make("/* annotation */ SELECT 1");
const call22 = make("/* annotation */ SELECT 1");
const call23 = make("/* annotation */ SELECT 1");
const call24 = make("/* annotation */ SELECT 1");
const call25 = make("/* annotation */ SELECT 1");
const call26 = make("/* annotation */ SELECT 1");
const call27 = make("/* annotation */ SELECT 1");
const call28 = make("/* annotation */ SELECT 1");
const call29 = make("/* annotation */ SELECT 1");
const call30 = make("/* annotation */ SELECT 1");
const call31 = make("/* annotation */ SELECT 1");
const result = call31();
