const ellipsisContent = <g stroke="currentColor" fill="currentColor">
  <path d="M8 256a56 56 0 1 1 112 0A56 56 0 1 1 8 256zm160 0a56 56 0 1 1 112 0 56 56 0 1 1 -112 0zm216-56a56 56 0 1 1 0 112 56 56 0 1 1 0-112z"></path>
</g>;

export const EllipsisIcon = () => <svg version="1.1" className="chakra-icon" xmlns="http://www.w3.org/2000/svg" x="0px" y="0px"
  width="9.5px" height="9.5px" viewBox="0 0 448 512"
>
  {ellipsisContent}
</svg>;

export const LargeEllipsisIcon = () => <svg version="1.1" className="chakra-icon" xmlns="http://www.w3.org/2000/svg" x="0px" y="0px"
  width="16px" height="16px" viewBox="0 0 448 512"
>
  {ellipsisContent}
</svg>; 