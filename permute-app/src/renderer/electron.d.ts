import { IPermuteState } from 'permute-node';

declare global {
  interface Window {
    Electron: {
      ipcRenderer: {
        getState(): Promise<IPermuteState>;
        openOutputDialog(callback: (args: [string]) => void): void;
        runProcessor(updateFn: (state: IPermuteState) => void, completeFn: (state: IPermuteState) => void): void;
        reverseFile(updateFn: (state: IPermuteState) => void, completeFn: (state: IPermuteState) => void, file: string): void;
        trimFile(updateFn: (state: IPermuteState) => void, completeFn: (state: IPermuteState) => void, file: string): void;
        addFile(file: string): void;
        removeFile(file: string): void;
        addProcessor(name: string): void;
        removeProcessor(name: string): void;
        setOutput(output: string): void;
        setDepth(depth: number): void;
        setPermutations(permutations: number): void;
        setNormalised(normalised: boolean): void;
        setTrimAll(trimAll: boolean): void;
        setInputTrail(trail: number): void;
        setOutputTrail(trail: number): void;
        showFile(file: string): void;
        deleteOutputFile(file: string): void;
        deleteAllOutputFiles(): void;
        cancel(): void;
        setCreateSubdirectories(createSubfolders: boolean): void;
      };
    };
  }
}

export { }; 