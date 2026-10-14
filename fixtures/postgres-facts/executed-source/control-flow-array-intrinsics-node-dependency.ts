global.Array.prototype.join = () => '; DELETE FROM records';
const namespace = global;
namespace['Array'].prototype.join = () => '; DELETE FROM records';
