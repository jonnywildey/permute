export const processorCategories = {
  'Time/Pitch': ['Half Speed', 'Double Speed', 'Random Pitch', 'Granular Stretch', 'Blur Stretch', 'Reverse'],
  'Modulation': ['Wow', 'Flutter', 'Chorus', 'Flange', 'Phaser', 'Tremolo', 'Lazer'],
  'Filter/Drive': ['Fuzz', 'Saturate', 'LFO Filter', 'Line Filter', 'Filter'],
  'Delay/Reverb': ['Metallic Delay', 'Rhythmic Delay', 'Reverb'],
  'Cross Processing': ['Cross Gain', 'Cross Filter', 'Cross Distort']
};

export const processorDescriptions: Record<string, React.ReactNode> = {
  Fuzz: `A distortion that ranges from gentle overdrive to fuzz to "running low on battery" sounds`,
  Saturate: `A fairly gentle, smooth overdrive.`,
  Reverse: `Reverses the audio. Best used in conjunction with other effects like delay`,
  'Granular Stretch': `Author's guess of how vintage samplers' time stretching works, cuts audio into small chunks, or grains, of sound and loops them. 
  Cycle length, crossfade and stretch amount are randomised. 
  Be careful when using this with high depths, it can dramatically increase the length of the audio`,
  'Blur Stretch': `A time stretch that uses a blurring algorithm to stretch the audio. Be careful with high depths, it can dramatically increase the length of the audio`,
  'Metallic Delay': `A delay with low duration (less than 100ms) and high feedback to create metallic sounds.`,
  'Rhythmic Delay': `A delay between 100ms and 1 second.`,
  'Filter': `Random Lo-pass, hi-pass, band-pass filters.`,
  Reverb: `A very 80s sounding reverb. Reverb length and mix are randomised`,
  'Half-Speed': `Converts the audio to half speed, lowering the pitch by an octave. The duration of the audio will change. There are no randomised parameters`,
  'Double Speed':
    'Converts the audio to double speed, increasing the pitch by an octave. The duration of the audio will change. There are no randomised parameters',
  'Random Pitch':
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
  'Cross Gain': `Modulates the gain of the audio based on the RMS energy of another audio file. 
    Modulating Audio file is stretched to the length of the original audio.
    Cross effects run quite slow. Be careful with high depth values or large numbers of files`,
  'Cross Filter': `Modulates a filter's frequency based on the RMS energy of another audio file. 
  Modulating Audio file is stretched to the length of the original audio.
  Cross effects run quite slow. Be careful with high depth values or large numbers of files`,
  'Cross Distort': `Modulates the distortion amount based on the RMS energy of another audio file.
  Uses a variety of gentler distortion algorithms including hyperbolic tangent, arctangent, soft clipping, and saturation.
  The distortion factor varies based on the modulating audio's amplitude.
  Cross effects run quite slow. Be careful with high depth values or large numbers of files`,
};
