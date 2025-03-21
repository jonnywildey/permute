import { Box, GridItem, Heading, IconButton, useColorMode } from '@chakra-ui/react';
import React, { useContext, useEffect, useState } from 'react';
import { AudioContext } from './AudioContext';
import { LargePlayIcon } from './icons/PlayIcon';
import { LargePauseIcon } from './icons/PauseIcon';
import { LargeStopIcon } from './icons/StopIcon';

export const AudioPlayer: React.FC = () => {
  const { colorMode } = useColorMode();
  const { resume, pause, stop, file, setOnPlayUpdate, isPlaying, setPosition } =
    useContext(AudioContext);
  const [secs, setSecs] = useState<number>(0);

  useEffect(() => {
    setOnPlayUpdate((s) => setSecs(s));
  }, []);

  let progress = (secs / file.durationSec) * 100;
  progress = progress > 100 ? 100 : progress;

  if (!file.name) {
    return (
      <GridItem rowSpan={2} colSpan={3} padding={3} mr={2} pos="relative" />
    );
  }

  const onClick: React.DragEventHandler<HTMLDivElement> = (ev) => {
    const x = ev.nativeEvent.offsetX;
    const width = document.getElementById('audio-image')!.offsetWidth;
    const progress = x / width;
    const newSecs = file.durationSec * progress;
    setPosition(newSecs);
    ev.stopPropagation();
  };
  const allowDrop: React.DragEventHandler<HTMLDivElement> = (ev) => {
    ev.preventDefault();
  };

  return (
    <GridItem rowSpan={2} colSpan={3} padding={3} mr={2} pos="relative">
      <Box pl={2} pr={2} marginBottom="-70px" width="100%">
        <Box
          width="100%"
          className="play-image"
          onDragOver={allowDrop}
          id="audio-image"
          dangerouslySetInnerHTML={{ __html: file.image }}
          onClick={onClick}
        />
        <Box
          bg="brand.126"
          className="audio-position"
          pos="relative"
          onClick={onClick}
          bottom="70px"
          height="70px"
          onDragOver={allowDrop}
          fontSize="lg"
          borderRight="1px solid"
          borderRightColor="brand.100"
          width={`${progress}%`}
        >
          &nbsp;
        </Box>
      </Box>
      <Box display="flex" alignItems="baseline" pt={2.5} pr={1}>
        <Heading size="md" pl={2} pr={2} width="100%" color={colorMode === 'dark' ? 'gray.50' : 'gray.800'}>
          {file.name}
        </Heading>

        <IconButton
          aria-label="show"
          variant="ghost"
          rounded="full"
          size="xs"
          icon={isPlaying ? <LargePauseIcon /> : <LargePlayIcon />}
          onClick={() => (isPlaying ? pause() : resume())}
          color="brand.5600"
          _hover={{ bg: 'brand.50' }}
        />
        <IconButton
          aria-label="show"
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
