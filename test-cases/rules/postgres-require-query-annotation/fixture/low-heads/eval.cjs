const named = "/* outer lexical */ SELECT 1";
function dynamic() {
  eval("var named = 'SELECT 2'");
  read(named); // low: false
  read("/* direct remains syntax proof */ SELECT 1"); // low: true
}
// The conservative file-level eval gap rejects const proof outside it too.
read(named); // low: false
