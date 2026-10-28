import { pool as p } from '@example/db';

p.query(); // no SQL argument is outside member advice
p.query({ ...{ text: 'DROP TABLE app.spread' } }); // spread has no direct text field
p.query({ 'text': 'DROP TABLE app.quoted_key' }); // quoted text key stays advisory
p.query({ 0: 'DROP TABLE app.numeric_key' }); // numeric key is not text
