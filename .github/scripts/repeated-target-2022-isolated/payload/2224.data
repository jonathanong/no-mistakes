export interface ServerContractsReport {
  routes: ServerRouteContract[];
  clientRefs: ServerContractClientRef[];
  mismatches: ServerContractMismatch[];
}

export interface ServerRouteContract {
  file: string;
  line: number;
  method: string;
  route: string;
  queryParams: string[];
}

export interface ServerContractClientRef {
  file: string;
  line: number;
  route: string;
  queryParams: string[];
  matchedRoute?: string | null;
}

export interface ServerContractMismatch {
  file: string;
  line: number;
  route: string;
  matchedRoute: string;
  missingParams: string[];
}
