const named = "/* outer lexical */ SELECT 1";
read(named); // low: true
// Dynamic ownership at the use site overrides an otherwise valid outer const.
with (object) {
  read(named); // low: false
  read("/* direct inside dynamic scope */ SELECT 1"); // low: true
}
