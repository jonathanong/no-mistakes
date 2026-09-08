class Service {
  static run() {}
}

const ExpressionService = class {
  static run() {}
  instance() {}
};

function api() {
  function run() {}
}

Service.run();
ExpressionService.run();
ExpressionService.instance();
api.run();
