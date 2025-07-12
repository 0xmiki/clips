/**
 * Fetches and parses the English transcript for a YouTube video.
 * @param videoId The YouTube video ID
 * @returns The formatted transcript as a string
 */
export async function fetchTranscript(videoId: string): Promise<string> {
  if (!videoId) throw new Error("Video ID is required");

  // Always use backend for reliability
  return await fetchTranscriptBackend(videoId);
}

/**
 * Backend fallback for fetching transcript using yt-dlp
 * @param videoId The YouTube video ID
 * @returns The formatted transcript as a string
 */
async function fetchTranscriptBackend(videoId: string): Promise<string> {
  const { invoke } = await import("@tauri-apps/api/core");

  try {
    const transcript = (await invoke("fetch_transcript_backend", {
      videoId,
    })) as string;
    return transcript;
  } catch (error) {
    throw new Error(`Failed to fetch transcript from backend: ${error}`);
  }
}
