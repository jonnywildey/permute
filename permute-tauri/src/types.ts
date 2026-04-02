// Moved from permute-node/src/index.ts — now the single source of truth for TS types.

export interface IPermuteState {
  output: string;
  error: string;
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
  createSubdirectories: boolean;
  permutationOutputs: IPermutationOutput[];
  viewedWelcome: boolean;
  maxStretch: number;
}

export interface IPermutationInput {
  path: string;
  name: string;
  durationSec: number;
  image: string;
}

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

export type GetStateCallback = (state: IPermuteState) => void;
