const handler = () => {};
module.exports = handler;
exports.handler = handler;
module.exports.other = handler;
module["exports"] = handler;
module[handler] = handler;
module.filename = "okay";
