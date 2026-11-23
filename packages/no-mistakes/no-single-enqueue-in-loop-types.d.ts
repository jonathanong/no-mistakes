export interface NoSingleEnqueueInLoopTarget {
  module: string;
  export: string;
}

export interface NoSingleEnqueueInLoopOptions {
  single?: NoSingleEnqueueInLoopTarget[];
  bulk?: NoSingleEnqueueInLoopTarget[];
  singleExportPattern?: string;
  bulkExportPattern?: string;
  moduleGlobs?: string[];
}
