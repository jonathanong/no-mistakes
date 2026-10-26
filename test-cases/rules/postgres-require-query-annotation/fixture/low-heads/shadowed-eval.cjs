const named = "/* outer lexical */ SELECT 1";
function ordinary(eval) {
  eval("var named = 'SELECT 2'");
  read(named); // low: true
}
