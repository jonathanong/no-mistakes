import { Controller, Get } from "@nestjs/common";

const prefix = "users";
const path = ":id";

@Controller(prefix)
export class ComputedController {
  @Get(path)
  findOne() {}
}
