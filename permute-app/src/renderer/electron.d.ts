import { IpcRenderer } from 'electron';
import { IPermuteState } from 'permute-node';

declare global {
  interface Window {
    Electron: {
      ipcRenderer: {
        addFile(file: string): void;
        addProcessor(name: string): void;
        cancel(): void;
        deleteAllOutputFiles(): void;
        deleteOutputFile(file: string): void;
        deselectAllProcessors(): void;
        getFileStats(files: string[]): Promise<any>;
        getState(): Promise<IPermuteState>;
        openOutputDialog(callback: (args: [string]) => void): void;
        removeFile(file: string): void;
        removeProcessor(name: string): void;
        reverseFile(
          updateFn: (state: IPermuteState) => void,
          completeFn: (state: IPermuteState) => void,
          file: string
        ): void;
        runProcessor(
          updateFn: (state: IPermuteState) => void,
          completeFn: (state: IPermuteState) => void
        ): void;
        selectAllProcessors(): void;
        setCreateSubdirectories(createSubfolders: boolean): void;
        setDepth(depth: number): void;
        setInputTrail(trail: number): void;
        setNormalised(normalised: boolean): void;
        setOutput(output: string): void;
        setOutputTrail(trail: number): void;
        setPermutations(permutations: number): void;
        setTrimAll(trimAll: boolean): void;
        showFile(file: string): void;
        trimFile(
          updateFn: (state: IPermuteState) => void,
          completeFn: (state: IPermuteState) => void,
          file: string
        ): void;
      } & IpcRenderer;
    };
  }
}

export { }; 