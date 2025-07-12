// a way to fetch transcript from the frontend. But not used

function parseTranscript(xmlString: string): string {
  const parser = new DOMParser();
  const xmlDoc = parser.parseFromString(xmlString, "application/xml");
  const textElements = xmlDoc.getElementsByTagName("text");
  let formattedTranscript = "";

  Array.from(textElements).forEach((element: any) => {
    const startTime = parseFloat(element.getAttribute("start"));
    const startMinutes = Math.floor(startTime / 60);
    const startSeconds = (startTime % 60).toFixed(0).padStart(2, "0");
    const formattedStart = `${startMinutes}:${startSeconds}`;
    const text = element.textContent;
    formattedTranscript += `${formattedStart}\n${text}\n`;
  });
  return formattedTranscript;
}

// Helper to POST data
async function postData(url = "", data = {}) {
  const response = await fetch(url, {
    method: "POST",
    mode: "cors",
    cache: "no-cache",
    credentials: "same-origin",
    headers: {
      "Content-Type": "application/json",
      Accept: "application/json",
    },
    redirect: "follow",
    referrerPolicy: "no-referrer",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to fetch data");
  return response.json();
}

try {
  // Fetch transcript info

  const data = await postData(
    "https://release-youtubei.sandbox.googleapis.com/youtubei/v1/player",
    {
      videoId: videoId,
      context: {
        client: {
          userAgent:
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
          clientName: "WEB",
          clientVersion: "2.20230101",
        },
      },
    },
  );

  let transcriptText = "";
  const captions =
    data.captions?.playerCaptionsTracklistRenderer?.captionTracks;
  if (captions && captions.length > 0) {
    const englishCaption = captions.find(
      (caption: any) => caption.languageCode === "en",
    );
    if (englishCaption) {
      const transcriptUrl = englishCaption.baseUrl;
      const transcriptResponse = await fetch(transcriptUrl);
      const xmlTranscript = await transcriptResponse.text();
      transcriptText = parseTranscript(xmlTranscript);
    } else {
      throw new Error("No English transcript available for this video.");
    }
  } else {
    throw new Error("No captions available for this video.");
  }
  return transcriptText;
} catch (error) {
  console.warn(
    "Frontend transcript fetch failed, trying backend fallback:",
    error,
  );
  // Fallback to backend transcript fetching
  return await fetchTranscriptBackend(videoId);
}
