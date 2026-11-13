import {read} from "@bench/db";
import {named, missing, namedValue, missingValue, record, missingRecord, builder, plainText} from "./plain.mjs";
read(named()); // low: true
read(missing()); // low: false
read(namedValue); // low: true
read(missingValue); // low: false
const paired = record();
read(paired.text, paired.values); // low: true
const unmarked = missingRecord();
read(unmarked.text, unmarked.values); // low: false
const statement = builder();
statement.append(runtimeTail);
read(statement); // low: true
const plain = plainText();
read(plain); // low: false
const unsafe = plainText();
unsafe.append(runtimeTail);
read(unsafe); // low: false
import {named as absent} from "./not-prepared.mjs";
read(absent()); // low: false
import {make as broken} from "./broken.mjs";
read(broken()); // low: false
read(plainText()); // low: true
const {text, values} = record();
read(text, values); // low: true
const {text: builderText, values: builderValues} = builder();
read(builderText, builderValues); // low: false
const options = record();
read(options); // low: false
