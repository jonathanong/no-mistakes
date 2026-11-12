import {read} from "@bench/db";
import {healthy, plain, reexported, cyclic, unresolved, broken, replacement, suspended, untrusted} from "./factories.mjs";
const first = await healthy();
read(first); // low: true
const second = await plain();
read(second); // low: true
const third = await reexported();
read(third); // low: true
const fourth = await cyclic();
read(fourth); // low: false
const fifth = await unresolved();
read(fifth); // low: false
const sixth = await broken();
read(sixth); // low: false
const seventh = await replacement();
read(seventh); // low: false
const eighth = await suspended();
read(eighth); // low: false
const ninth = await untrusted();
read(ninth); // low: false
