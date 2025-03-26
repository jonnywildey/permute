declare module 'permute-node' {
  export interface IProcessorAttribute {
    key: string;
    value: string;
  }

  export interface IProcessor {
    name: string;
    attributes: IProcessorAttribute[];
  }

  export interface IPermutationOutput {
    path: string;
    progress: number;
    image: string;
    processors: IProcessor[];
    name: string;
    durationSec: number;
    deleted: boolean;
  }

  export interface IPermutationInput {
    path: string;
    name: string;
    image: string;
    durationSec: number;
  }

  export interface IPermuteState {
    output: string;
    error: string | null;
    processing: boolean;
    highSampleRate: boolean;
    inputTrail: number;
    outputTrail: number;
    files: IPermutationInput[];
    permutations: number;
    permutationDepth: number;
    processorCount: number;
    processorPool: string[];
    allProcessors: string[];
    normaliseAtEnd: boolean;
    trimAll: boolean;
    permutationOutputs: IPermutationOutput[];
    createSubdirectories: boolean;
    viewedWelcome: boolean;
  }
} 