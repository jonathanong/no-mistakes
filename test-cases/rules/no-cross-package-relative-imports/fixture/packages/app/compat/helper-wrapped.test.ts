import { client } from "@sample/external";
import { getMocks, mocks } from "./vi-helper";

export async function loadFixtures(specifier: string) {
  const fromReexport = await mocks.importActual("../../shared/src/status.mts");
  const fromFactory = await getMocks().importMock("../../shared/src/status.mts");
  const wrapped = getMocks();
  const fromLocal = await wrapped.importActual("../../shared/src/status.mts");
  const fromComputed = await wrapped.importMock(specifier);
  const samePackage = await wrapped.importActual("./a.ts");
  const packageName = await wrapped.importMock("@sample/shared");
  const externalClient = await client.importActual("../../shared/src/status.mts");
  class LocalLoader {
    static importActual(specifier: string) {
      return specifier;
    }
  }
  const localClient = LocalLoader.importActual("../../shared/src/status.mts");
  return {
    fromReexport, fromFactory, fromLocal, fromComputed, samePackage, packageName, externalClient, localClient,
  };
}
