class Alpha extends Beta {
  static run() {}
}
class Beta extends Alpha {}

function go() {
  Alpha.run();
  Alpha.missing();
}

go();
