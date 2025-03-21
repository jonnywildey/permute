import {
  Heading,
  Modal,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalOverlay,
  Text,
  useColorMode,
} from '@chakra-ui/react';
import { useEffect, useState } from 'react';

export interface WelcomeProps {
  isOpen: boolean;
  onClose: () => void;
}

export const Welcome: React.FC<WelcomeProps> = ({ isOpen, onClose }) => {
  const [isLoaded, setIsLoaded] = useState(false);
  const { colorMode } = useColorMode();

  useEffect(() => {
    // Preload both background images
    const lightImg = new Image();
    lightImg.src = require('../img/bg2.png');
    const darkImg = new Image();
    darkImg.src = require('../img/bgdark.png');

    // Wait for the appropriate image to load based on color mode
    const img = colorMode === 'dark' ? darkImg : lightImg;
    if (img.complete) {
      setIsLoaded(true);
    } else {
      img.onload = () => setIsLoaded(true);
    }
  }, [colorMode]);

  return (
    <Modal onClose={onClose} isOpen={isOpen}>
      <ModalOverlay
        bg="blackAlpha.300"
        transition="all 0.3s"
      />
      <ModalContent
        className="modal"
        opacity={isLoaded ? 1 : 0}
        transform={isLoaded ? "translateY(0)" : "translateY(20px)"}
        transition="all 0.3s ease-out"
        style={{
          backgroundImage: `url(${colorMode === 'dark' ? require('../img/bgdark.png') : require('../img/bg2.png')}) !important`
        }}
      >
        <ModalCloseButton />
        <ModalBody>
          <Heading size="lg">Welcome to Permute!</Heading>
          <Text fontSize="xl" mb={2}>
            Permute utilises the power of <b>random permutations</b> to
            dramatically alter audio to something alien, abstract, and possibly{' '}
            <b>useful</b>.
          </Text>
          <Heading size="md">To start</Heading>
          <Text fontSize="xl" mb={4}>
            Drag an audio file (AIF or WAV) file (or multiple files) into the{' '}
            <b>Select Files area</b>. Select an <b>output directory</b>. Click{' '}
            <b>run</b>. Have fun!
          </Text>
          <Text fontSize="xl" mb={2}>
            Audio source files are run through random chains of processors with
            randomised parameters into an output directory. As every run is
            likely to produce different results it is often helpful to process
            the same file multiple times. This can be done by increasing the{' '}
            <i>permutations</i> field.
          </Text>
          <Text fontSize="xl" mb={2}>
            The <i>depth</i> controls how long the chain of processors should
            be. Depths of 3 or more tend to result in very abstract outputs
          </Text>
          <Text fontSize="xl" mb={2}>
            It is recommended you keep the <b>normalise</b> feature enabled,
            processors can easily cause digital clipping
          </Text>
        </ModalBody>
      </ModalContent>
    </Modal>
  );
};
