import { primary } from '@vendor/valkey';
const items = ['one'];
class WrongBase { constructor(client = primary) { client.get('key'); } }
class WrongDerived extends WrongBase {}
function constructWrongDerived() { return new WrongDerived(unrelated); } for (const item of items) { constructWrongDerived(); }
class DefaultBase { constructor(client = primary) { client.get('key'); } }
class DefaultDerived extends DefaultBase {}
function constructDefaultDerived() { return new DefaultDerived(); } for (const item of items) { constructDefaultDerived(); }
class KnownBase { constructor(client = primary) { client.get('key'); } }
class Middle extends KnownBase {}
class KnownDerived extends Middle {}
function constructKnownDerived() { return new KnownDerived(primary); } for (const item of items) { constructKnownDerived(); }
class ExplicitBase { constructor(client = primary) { client.get('key'); } }
class ExplicitDerived extends ExplicitBase { constructor(client = primary) { super(client); } }
function constructExplicitDerived() { return new ExplicitDerived(unrelated); } for (const item of items) { constructExplicitDerived(); }
class ExplicitDefaultBase { constructor(client = primary) { client.get('key'); } }
class ExplicitDefaultDerived extends ExplicitDefaultBase { constructor(client = primary) { super(client); } }
function constructExplicitDefaultDerived() { return new ExplicitDefaultDerived(); } for (const item of items) { constructExplicitDefaultDerived(); }
