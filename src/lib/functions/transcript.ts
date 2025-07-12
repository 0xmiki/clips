export function truncateTranscript(
  transcript: string,
  startSeconds: number,
  endSeconds: number,
): string {
  if (!transcript || (startSeconds === 0 && endSeconds === 0)) {
    return transcript;
  }

  const lines = transcript.split("\n");
  const truncatedLines: string[] = [];
  let currentTime = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (!line) continue;

    // Check if this line contains a timestamp (MM:SS format)
    const timeMatch = line.match(/^(\d{1,2}):(\d{2})$/);
    if (timeMatch) {
      const minutes = parseInt(timeMatch[1]);
      const seconds = parseInt(timeMatch[2]);
      currentTime = minutes * 60 + seconds;

      // If we're within the time range, include this timestamp
      if (currentTime >= startSeconds && currentTime <= endSeconds) {
        truncatedLines.push(line);
      }
    } else {
      // This is a text line, include it if we're within the time range
      if (currentTime >= startSeconds && currentTime <= endSeconds) {
        truncatedLines.push(line);
      }
    }
  }

  return truncatedLines.join("\n");
}
