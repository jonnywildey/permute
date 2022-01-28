import React from "react";


export interface IAudioContext {
  playFile: (path: string) => void;
  pause: () => void;
}

export const AudioContext = React.createContext<IAudioContext>({} as any);

export const CreateAudioContext: React.FC = ({ children }) => {
  const audioEl = React.createRef<HTMLAudioElement>();
  
  const playFile = (path: string) => {
    const audio = audioEl.current!;
    audio.src = `audio:///${path}`;
  }

  const pause = () => {
    const audio = audioEl.current!;
    audio.pause();
  }

  const value: IAudioContext = {
    playFile,
    pause,
  };

  return <AudioContext.Provider value={value}>
    <audio
      autoPlay={true}
        ref={audioEl}
      >Nope</audio>
    {children}
    </AudioContext.Provider>;

}