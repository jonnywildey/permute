export const processorDescriptions: Record<string, React.ReactNode> = {
  Fuzz: `A distortion that ranges from gentle overdrive to fuzz to "running low on battery" sounds`,
  Saturate: `A gentle, smooth overdrive. This only has one setting, it has no randomised parameters`,
  Reverse: `Reverses the audio. Best used in conjunction with other effects like delay`,
  'Granular stretch': `Inspired by what the audio guesses is how vintage samplers' time stretching works, 
  applies a granular time stretch. 
  Cycle length, crossfade and stretch amount are randomised. 
  Be careful when using this with high depths, it can create very long files`,
  'Metallic delay': `A delay with low duration (less than 100ms) and high feedbacks to create metallic sounds.`,
  'Rhythmic delay': `A delay between 100ms and 1 second.`,
  Reverb: `A very 80s sounding reverb. Reverb length and mix are randomised`,
  'Half-Speed': `Converts the audio to half speed, lowering the pitch by an octave. The duration of the audio will change. There are no randomised parameters`,
  'Double speed':
    'Converts the audio to double speed, increasing the pitch by an octave. The duration of the audio will change. There are no randomised parameters',
  'Random pitch':
    'Shifts the pitch of the audio by a random interval. The duration of the audio will change',
  Wow: 'A low speed vibrato, high depth effect similar to a warped record. Depth, speed and mix levels are randomised',
  Flutter: `A high speed vibrato effect, giving a warbley or fluttery sound.
  Can sound similar to tremolo. Depth, speed and mix levels are randomised`,
  Chorus:
    'A thickening chorus effect. Depth, speed and mix levels are randomised',
  Flange: `A "zero-through" flange effect. Speed and depth are randomised`,
  Phaser: 'A basic phaser effect. Speed and depth are randomised',
  Tremolo: `Low-ish frequency amplitude modulation. Weirdly good for percussive sounds`,
  Lazer: `Amplitude modulation where the frequency is determined by strength of signal. 
  Good for robot sounds and adding high frequency elements to a sound`,
  'LFO Filter': `A low-frequency oscillator that controls the cutoff frequency of a filter.`,
  'Line Filter': `Filter frequency increases (or decreases) over the length of the audio.`,
};
