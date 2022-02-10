import React, { useState } from 'react';
import type { IPermutationInput } from 'permute-node';

export interface IAudioContext {
  playFile: (file: IPermutationInput) => void;
  pause: () => void;
  stop: () => void;
  resume: () => void;
  setOnPlayUpdate: (cb: (secs: number) => void) => void;
  setPosition: (secs: number) => void;
  isPlaying: boolean;
  file: IPermutationInput;
  reset: () => void;
}

export interface IAudioState {
  file: IPermutationInput;
  isPlaying: boolean;
  timeChecker?: NodeJS.Timer;
  onPlayUpdate?: (secs: number) => void;
  audio: HTMLAudioElement;
}

const defaultFile: IPermutationInput = {
  path: '',
  name: '',
  image: '',
  durationSec: 0,
};

export const AudioContext = React.createContext<IAudioContext>({} as any);

const UPDATE_LATENCY = 50;

export const CreateAudioContext: React.FC = ({ children }) => {
  const [state, setState] = useState<IAudioState>({
    file: defaultFile,
    isPlaying: false,
    audio: new Audio(),
  });

  const playFile = (file: IPermutationInput) => {
    clearInterval(state.timeChecker!);
    const { path } = file;
    state.audio.autoplay = true;
    // Add date to string to avoid caching
    state.audio.src = `audio:///${new Date().toISOString()}/${path}`;
    const newState = { ...state, isPlaying: true, file };

    let startedPlaying = false;
    const timeChecker = setInterval(() => {
      if (state.audio.paused && startedPlaying) {
        setState({ ...newState, isPlaying: false, timeChecker: undefined });
        clearInterval(timeChecker);
      }
      startedPlaying = state.audio.currentTime != 0 && startedPlaying === false;

      if (state.onPlayUpdate) {
        state.onPlayUpdate(state.audio.currentTime);
      }
    }, UPDATE_LATENCY);
    setState({ ...newState, timeChecker });
  };

  const resume = () => {
    if (!state.file.path) {
      return;
    }
    clearInterval(state.timeChecker!);
    state.audio.play();

    let startedPlaying = false;
    const timeChecker = setInterval(() => {
      if (state.audio.paused && startedPlaying) {
        setState({ ...state, isPlaying: false, timeChecker: undefined });
        clearInterval(timeChecker);
      }
      startedPlaying = state.audio.currentTime != 0 && startedPlaying === false;
      if (state.onPlayUpdate) {
        state.onPlayUpdate(state.audio.currentTime);
      }
    }, UPDATE_LATENCY);
    setState({ ...state, isPlaying: true, timeChecker });
  };

  const pause = () => {
    // const audio = audioEl.current!;
    state.audio.pause();
    setState({ ...state, isPlaying: false });
  };

  const stop = () => {
    // const audio = audioEl.current!;
    state.audio.pause();
    state.audio.currentTime = 0;
    if (state.onPlayUpdate) {
      state.onPlayUpdate(0);
    }
    setState({ ...state, isPlaying: false });
  };

  const reset = () => {
    clearInterval(state.timeChecker!);
    state.audio.pause();
    state.audio.currentTime = 0;
    if (state.onPlayUpdate) {
      state.onPlayUpdate(0);
    }
    setState({
      ...state,
      file: defaultFile,
      isPlaying: false,
      audio: new Audio(),
    });
  };

  const setOnPlayUpdate = (cb: (secs: number) => void) => {
    setState({ ...state, onPlayUpdate: cb });
  };

  const setPosition = (secs: number) => {
    state.audio.currentTime = secs;
    if (state.onPlayUpdate) {
      state.onPlayUpdate(secs);
    }
  };

  const value: IAudioContext = {
    playFile,
    setOnPlayUpdate,
    isPlaying: state.isPlaying,
    setPosition,
    resume,
    pause,
    stop,
    reset,
    file: state.file,
  };

  return (
    <AudioContext.Provider value={value}>{children}</AudioContext.Provider>
  );
};
