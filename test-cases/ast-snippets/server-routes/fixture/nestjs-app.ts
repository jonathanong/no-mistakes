import { Controller, Get, Post, Put } from "@nestjs/common";

@Controller("users")
export class UsersController {
  @Get()
  findAll() {}

  @Get(":id")
  findOne() {}

  @Post()
  create() {}
}

@Controller({ path: "health" })
export class HealthController {
  @Get()
  check() {}
}

@Controller()
export class RootController {
  @Put("ready")
  ready() {}
}

const { Controller: CjsController, Delete } = require("@nestjs/common");

@CjsController("cjs")
export class CjsControllerClass {
  @Delete("gone")
  remove() {}
}
