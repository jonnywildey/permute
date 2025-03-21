import play from '../../img/icons/play-button.svg';

const playContent = <g fill="currentColor">
  <g>
    <path d="M204.11,0C91.388,0,0,91.388,0,204.111c0,112.725,91.388,204.11,204.11,204.11c112.729,0,204.11-91.385,204.11-204.11
			C408.221,91.388,316.839,0,204.11,0z M286.547,229.971l-126.368,72.471c-17.003,9.75-30.781,1.763-30.781-17.834V140.012
			c0-19.602,13.777-27.575,30.781-17.827l126.368,72.466C303.551,204.403,303.551,220.217,286.547,229.971z"/>
  </g>
</g>

export const PlayIcon = () => <svg version="1.1" className="chakra-icon" id="Capa_1" xmlns="http://www.w3.org/2000/svg" x="0px" y="0px"
  width="9.5px" height="9.5px" viewBox="0 0 408.221 408.221"
>
  {playContent}
</svg>
  ;
export const LargePlayIcon = () => <svg version="1.1" className="chakra-icon" id="Capa_1" xmlns="http://www.w3.org/2000/svg" x="0px" y="0px"
  width="16px" height="16px" viewBox="0 0 408.221 408.221"
>
  {playContent}
</svg>
  ;
