import { Box, GridItem, Heading, IconButton, useColorMode, Tooltip } from '@chakra-ui/react';
import React, { useContext, useEffect, useRef } from 'react';
import { AudioContext } from './AudioContext';
import { LargePlayIcon } from './icons/PlayIcon';
import { LargePauseIcon } from './icons/PauseIcon';
import { LargeStopIcon } from './icons/StopIcon';

const AUDIO_PLAYER_TOOLTIP_DELAY = 1400;

export const AudioPlayer: React.FC = () => {
  const { colorMode } = useColorMode();
  const { resume, pause, stop, file, getCurrentTime, isPlaying, setPosition } =
    useContext(AudioContext);

  // Direct ref to the progress bar DOM node — we animate its width without
  // going through React state, so there are zero re-renders during playback.
  const progressRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // scaleX(fraction) is GPU-composited — no layout reflow per frame.
    const setScale = (fraction: number) => {
      if (progressRef.current) {
        progressRef.current.style.transform = `scaleX(${Math.min(fraction, 1)})`;
      }
    };

    if (!isPlaying) {
      setScale(file.durationSec > 0 ? getCurrentTime() / file.durationSec : 0);
      return;
    }

    let raf: number;
    const tick = () => {
      if (file.durationSec > 0) setScale(getCurrentTime() / file.durationSec);
      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(raf);
  }, [isPlaying, file.durationSec, getCurrentTime]);

  if (!file.name) {
    return <GridItem rowSpan={2} colSpan={3} padding={3} mr={2} pos="relative" />;
  }

  const handleClick: React.MouseEventHandler<HTMLDivElement> = (ev) => {
    const x = ev.nativeEvent.offsetX;
    const width = document.getElementById('audio-image')!.offsetWidth;
    const fraction = x / width;
    const newSecs = file.durationSec * fraction;
    setPosition(newSecs);
    // Update the bar immediately so it doesn't wait for the next rAF tick.
    if (progressRef.current) {
      progressRef.current.style.transform = `scaleX(${Math.min(fraction, 1)})`;
    }
  };

  const allowDrop: React.DragEventHandler<HTMLDivElement> = (ev) => ev.preventDefault();

  return (
    <GridItem rowSpan={2} colSpan={3} padding={3} mr={2} pos="relative">
      <Box pl={2} pr={2} marginBottom="-70px" width="100%">
        <Box
          width="100%"
          className="play-image"
          onDragOver={allowDrop}
          id="audio-image"
          dangerouslySetInnerHTML={{ __html: file.image }}
          onClick={handleClick}
        />
        {/* Width is controlled via the ref during playback; the initial value is
            read from getCurrentTime() so re-renders (e.g. when isPlaying flips)
            don't cause a flash back to 0% before the next rAF tick. */}
        <Box
          ref={progressRef}
          bg="brand.126"
          className="audio-position"
          pos="relative"
          bottom="70px"
          height="70px"
          fontSize="lg"
          borderRight="1px solid"
          borderRightColor="brand.100"
          pointerEvents="none"
          style={{
            width: '100%',
            transformOrigin: 'left center',
            transform: `scaleX(${file.durationSec > 0 ? Math.min(getCurrentTime() / file.durationSec, 1) : 0})`,
            willChange: 'transform',
          }}
        >
          &nbsp;
        </Box>
      </Box>
      <Box display="flex" alignItems="baseline" pt={2.5} pr={1}>
        <Tooltip openDelay={AUDIO_PLAYER_TOOLTIP_DELAY} label={file.name}>
          <Heading
            size="md"
            pl={2}
            pr={2}
            mb={2}
            width="100%"
            color={colorMode === 'dark' ? 'gray.50' : 'gray.800'}
            className="filename-ellipsis"
          >
            {file.name}
          </Heading>
        </Tooltip>
        <IconButton
          aria-label="play/pause"
          variant="ghost"
          rounded="full"
          size="xs"
          icon={isPlaying ? <LargePauseIcon /> : <LargePlayIcon />}
          onClick={() => (isPlaying ? pause() : resume())}
          color="brand.5600"
          _hover={{ bg: 'brand.50' }}
        />
        <IconButton
          aria-label="stop"
          variant="ghost"
          rounded="full"
          size="xs"
          icon={<LargeStopIcon />}
          onClick={() => stop()}
          color="brand.5600"
          _hover={{ bg: 'brand.50' }}
        />
      </Box>
    </GridItem>
  );
};
