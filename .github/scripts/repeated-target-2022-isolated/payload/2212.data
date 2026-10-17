export interface SignatureImpactLocation {
  file: string;
  symbol: string;
  line: number;
  kind: string;
}

export interface SignatureImpactCaller {
  file: string;
  symbol?: string;
  depth: number;
  via: string[];
}

export interface SignatureImpactTest {
  file: string;
  depth: number;
  via: string[];
}

export interface SignatureImpactWarning {
  type: string;
  message: string;
}

export interface SignatureImpactResult {
  roots: string[];
  symbol: string;
  definition: SignatureImpactLocation;
  exports: SignatureImpactLocation[];
  productionCallers: SignatureImpactCaller[];
  testCallers: SignatureImpactCaller[];
  suggestedTests: SignatureImpactTest[];
  warnings: SignatureImpactWarning[];
}
