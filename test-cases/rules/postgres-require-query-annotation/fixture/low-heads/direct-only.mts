import { read } from "@bench/db";
read("/* literal */ SELECT 1"); // low: true
read(`/* template */ SELECT ${value}`); // low: true
