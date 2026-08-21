import { All, Controller, Delete, Get, Head, Options, Patch, Post, Put } from "@nestjs/common";

@Controller("users")
export class UsersController {
  @Get()
  findAll() {}

  @Get(":id")
  findOne() {}

  @Post()
  create() {}

  @Get("static")
  static hidden() {}
}

@Controller({ path: "health" })
export class HealthController {
  @Get()
  check() {}

  @Head()
  ping() {}
}

@Controller()
export class RootController {
  @Put("ready")
  ready() {}

  @Patch("ready")
  patchReady() {}

  @Options("ready")
  optionsReady() {}

  @All("any")
  any() {}
}

@Controller
export class BareController {
  @Get("bare")
  bare() {}
}

const { Controller: CjsController, Delete } = require("@nestjs/common");

@CjsController("cjs")
export class CjsControllerClass {
  @Delete("gone")
  remove() {}
}
