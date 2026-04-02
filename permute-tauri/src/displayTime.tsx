export const displayTime = (t: number): string => {
  const second_fraction = Math.round((t % 1) * 100);
  const seconds = Math.floor(t) % 60;
  const minutes = Math.floor(t / 60);
  const minutesStr =
    minutes === 0 ? '00' : minutes < 10 ? `0${minutes}` : minutes;
  const secondsStr = seconds < 10 ? `0${seconds}` : seconds;
  const fractionStr =
    second_fraction < 10 ? `0${second_fraction}` : second_fraction;
  return `${minutesStr}:${secondsStr}.${fractionStr}`;
};
