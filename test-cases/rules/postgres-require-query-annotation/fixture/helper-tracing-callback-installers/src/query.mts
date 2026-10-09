import { write } from "@app/db";

function objectFirst(value) {
  const consumed = opaque(arguments, () => {
    const installed = (arguments[0] = () => write("SELECT 1")); // finding:object-first
  });
}
const first = objectFirst("original");

function installerFirst(value) {
  const consumed = opaque(() => {
    const installed = (arguments[0] = () => write("SELECT 2")); // finding:installer-first
  }, arguments);
}
const reversed = installerFirst("original");

function cyclic(value) {
  // A container can contain its own identity without causing repeated traversal.
  const cycle = (arguments[0] = arguments);
  const consumed = opaque(arguments, () => {
    const installed = (arguments[1] = () => write("SELECT 3")); // finding:cyclic-container
  });
}
const cycleResult = cyclic("original");

function duplicateInstaller(value) {
  const installer = () => {
    write("SELECT 5"); // finding:duplicate-installer-body
    const installed = (arguments[0] = () => write("SELECT 4")); // finding:duplicate-installed-callback
  };
  const consumed = opaque(arguments, installer, installer);
}
const duplicate = duplicateInstaller("original");
