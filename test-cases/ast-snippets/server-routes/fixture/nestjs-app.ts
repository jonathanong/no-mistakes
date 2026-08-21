import { All, Controller, Delete, Get, Head, Header, Injectable, Options, Patch, Post, Put } from "@nestjs/common";

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

  @Header("X-Test", "1")
  @Get("hdr")
  hdr() {}
}

@Controller({ path: "health" })
export class HealthController {
  @Get()
  check() {}

  @Head()
  ping() {}
}

@Controller({ host: "localhost" })
export class HostController {
  @Get("host")
  host() {}
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

  get ignored() {
    return 1;
  }
}

const { Controller: CjsController, Delete } = require("@nestjs/common");

@CjsController("cjs")
export class CjsControllerClass {
  @Delete("gone")
  remove() {}
}

@Injectable()
@Controller("inj")
export class InjController {
  @Get()
  inj() {}
}

function extra(_target: unknown) {}

@extra
@Controller("extra")
export class ExtraController {
  @Get()
  extra() {}
}

@Controller()
export default class {
  @Get("anon")
  anon() {}
}
