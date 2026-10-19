import { query } from "@example/db";
declare const tail: string;
query(`SELECT ${tail}`); // The leading SQL proves the violation; the tail remains opaque.
