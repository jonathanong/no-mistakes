import { read } from "@bench/db";
import sql from "sql-template-strings";

function closed(name: string) {
  
  const query = sql``.append(`/* closed:${name} */ SELECT 1`);
  read(query); // low: true
}
function caller_closed() { const select = (name: string) => closed(name); select("one"); }

function twoNames(name: string) {
  
  const query = sql``.append(`/* twoNames:${name} */ SELECT 1`);
  read(query); // low: true
}
function caller_twoNames() { const select = (name: string) => twoNames(name); select("one"); select("two"); }

function duplicateName(name: string) {
  
  const query = sql``.append(`/* duplicateName:${name} */ SELECT 1`);
  read(query); // low: true
}
function caller_duplicateName() { const select = (name: string) => duplicateName(name); select("one"); select("one"); }

function blank(name: string) {
  
  const query = sql``.append(`/* blank:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_blank() { const select = (name: string) => blank(name); select("   "); }

function emptyCall(name: string) {
  
  const query = sql``.append(`/* emptyCall:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_emptyCall() { const select = (name: string) => emptyCall(name); select(); }

function extraArgs(name: string) {
  
  const query = sql``.append(`/* extraArgs:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_extraArgs() { const select = (name: string) => extraArgs(name); select("one", "two"); }

function computedArg(name: string) {
  
  const query = sql``.append(`/* computedArg:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_computedArg() { const select = (name: string) => computedArg(name); select("one" + "two"); }

function templateArg(name: string) {
  
  const query = sql``.append(`/* templateArg:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_templateArg() { const select = (name: string) => templateArg(name); select(`one`); }

function wrongContext(name: string) {
  
  const query = sql``.append(`/* wrongContext:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_wrongContext() { const select = (name: string) => wrongContext(name); const invoke = () => select("one"); invoke(); }

function selectorEscape(name: string) {
  
  const query = sql``.append(`/* selectorEscape:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_selectorEscape() { const select = (name: string) => selectorEscape(name); opaque(select); select("one"); }

function helperEscape(name: string) {
  
  const query = sql``.append(`/* helperEscape:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_helperEscape() { const select = (name: string) => helperEscape(name); opaque(helperEscape); select("one"); }

function missingHelperArg(name: string) {
  
  const query = sql``.append(`/* missingHelperArg:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_missingHelperArg() { const select = (name: string) => missingHelperArg(); select("one"); }

function twoHelperCalls(name: string) {
  
  const query = sql``.append(`/* twoHelperCalls:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_twoHelperCalls() { const select = (name: string) => { twoHelperCalls(name); return twoHelperCalls(name); }; select("one"); }

function helperReassigned(name: string) {
  
  const query = sql``.append(`/* helperReassigned:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_helperReassigned() { const select = (name: string) => helperReassigned(name); helperReassigned = other; select("one"); }

function parameterReassigned(name: string) {
  name = dynamicName;
  const query = sql``.append(`/* parameterReassigned:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_parameterReassigned() { const select = (name: string) => parameterReassigned(name); select("one"); }

function tooManyNames(name: string) {
  
  const query = sql``.append(`/* tooManyNames:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_tooManyNames() { const select = (name: string) => tooManyNames(name); select("n0");select("n1");select("n2");select("n3");select("n4");select("n5");select("n6");select("n7");select("n8");select("n9");select("n10");select("n11");select("n12");select("n13");select("n14");select("n15");select("n16"); }

function untrustedTag(name: string) {
  
  const query = untrusted``.append(`/* untrustedTag:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_untrustedTag() { const select = (name: string) => untrustedTag(name); select("one"); }

function differentParameter(name: string) {
  
  const query = sql``.append(`/* differentParameter:${name} */ SELECT 1`);
  read(query); // low: false
}
function caller_differentParameter() { const select = (name: string, other: string) => differentParameter(other); select("one"); }
