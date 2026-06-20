import React, { useState, useRef, useCallback, useMemo } from 'react';
import { convertFileSrc } from '@tauri-apps/api/core';
import type { IPermutationInput } from './types';

export interface IAudioContext {
  playFile: (file: IPermutationInput) => void;
  pause: () => void;
  stop: () => void;
  resume: () => void;
  getCurrentTime: () => number;
  setPosition: (secs: number) => void;
  isPlaying: boolean;
  file: IPermutationInput;
  reset: () => void;
}

// Kept for external compatibility
export interface IAudioState {
  file: IPermutationInput;
  isPlaying: boolean;
  timeChecker?: ReturnType<typeof setInterval>;
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

export const CreateAudioContext: React.FC<{ children?: React.ReactNode }> = ({ children }) => {
  const [isPlaying, setIsPlaying] = useState(false);
  const [file, setFile] = useState<IPermutationInput>(defaultFile);
  const audioRef = useRef(new Audio());

  const playFile = useCallback((f: IPermutationInput) => {
    const audio = audioRef.current;
    audio.pause();
    audio.src = convertFileSrc(f.path) + '?t=' + Date.now();
    audio.onended = () => setIsPlaying(false);
    setFile(f);
    // Explicit play() rather than autoplay=true: the returned Promise resolves
    // only when the browser has actually started playback (after buffering).
    // Delaying setIsPlaying(true) until then prevents the rAF loop from running
    // while getCurrentTime() is stuck at 0, which caused the ~500ms stutter.
    audio.play().then(() => setIsPlaying(true)).catch(console.error);
  }, []);

  const resume = useCallback(() => {
    if (!audioRef.current.src) return;
    audioRef.current.play();
    setIsPlaying(true);
  }, []);

  const pause = useCallback(() => {
    audioRef.current.pause();
    setIsPlaying(false);
  }, []);

  const stop = useCallback(() => {
    const audio = audioRef.current;
    audio.pause();
    audio.currentTime = 0;
    setIsPlaying(false);
  }, []);

  const reset = useCallback(() => {
    const audio = audioRef.current;
    audio.pause();
    audio.onended = null;
    audio.currentTime = 0;
    audioRef.current = new Audio();
    setFile(defaultFile);
    setIsPlaying(false);
  }, []);

  // Stable getter — AudioPlayer polls this via requestAnimationFrame instead of
  // receiving push callbacks, eliminating React state updates during playback.
  const getCurrentTime = useCallback(() => audioRef.current.currentTime, []);

  const setPosition = useCallback((secs: number) => {
    audioRef.current.currentTime = secs;
  }, []);

  const value = useMemo<IAudioContext>(() => ({
    playFile,
    getCurrentTime,
    isPlaying,
    setPosition,
    resume,
    pause,
    stop,
    reset,
    file,
  }), [file, isPlaying, pause, playFile, reset, resume, getCurrentTime, setPosition, stop]);

  return (
    <AudioContext.Provider value={value}>{children}</AudioContext.Provider>
  );
};
