export function timeToSeconds(ts: string): number {
  const parts = ts.split(":").map(Number).reverse();
  let seconds = 0;
  for (let i = 0; i < parts.length; i++) {
    seconds += parts[i] * Math.pow(60, i);
  }
  return seconds;
}

export function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  if (h > 0)
    return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`;
  return `${m}:${s.toString().padStart(2, "0")}`;
}
