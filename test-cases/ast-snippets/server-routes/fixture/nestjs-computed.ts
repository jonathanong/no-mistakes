import { Controller, Get } from "@nestjs/common";
import * as Nest from "@nestjs/common";

const prefix = "users";
const path = ":id";

@Controller(prefix)
export class ComputedController {
  @Get(path)
  findOne() {}
}

const options = { path: "hidden" };

@Controller({ ...options })
export class SpreadController {
  @Get()
  hidden() {}
}

@Controller({ path: prefix })
export class ComputedObjectController {
  @Get()
  hidden() {}
}

@Nest.Controller("ns")
export class NamespaceController {
  @Nest.Get()
  nested() {}
}
