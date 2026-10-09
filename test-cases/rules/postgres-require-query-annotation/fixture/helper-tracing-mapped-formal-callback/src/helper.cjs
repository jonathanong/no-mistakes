const database = require("@app/db");

function setup() {
  function installAfterEscape(statement, callback) {
    const escaped = opaque(arguments);
    var callback = () => database.query(statement); // finding:formal-write-after-escape
  }

  function installLiteralAfterEscape(callback) {
    const escaped = opaque(arguments);
    var callback = () => database.query("SELECT 4"); // finding:literal-formal-write-after-escape
  }

  function installWithoutEscape(statement, callback) {
    var callback = () => unknownMutation(statement);
    return statement;
  }

  function installForSecondConsumer(statement, callback) {
    const firstConsumer = opaque(arguments);
    var callback = () => database.query(statement); // finding:second-consumer
    const secondConsumer = opaque(arguments);
  }

  installAfterEscape("SELECT 1", "unused");
  installLiteralAfterEscape("unused");
  installForSecondConsumer("SELECT 3", "unused");
  database.query(installWithoutEscape("/* no escape control */ SELECT 2", "unused")); // known:noescape-control
}

setup();
