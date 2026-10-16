export interface MermaidValidationOptions {
  content: string;
  file?: string;
}

export type MermaidValidationDiagnosticCode = "invalid-syntax" | "unclosed-fence";

export interface MermaidValidationDiagnostic {
  code: MermaidValidationDiagnosticCode;
  file: string;
  fenceLine: number;
  diagramLine?: number;
  diagramColumn?: number;
  diagramType?: string;
  message: string;
}

export interface MermaidValidationResult {
  valid: boolean;
  diagramCount: number;
  diagnostics: MermaidValidationDiagnostic[];
}
