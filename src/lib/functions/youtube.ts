export const YOUTUBE_URL_REGEX = new RegExp(
  `^
    (?:
        (?:https?://|//)
        (?:
            (?:(?:(?:\\w+\\.)?[yY][oO][uU][tT][uU][bB][eE](?:-nocookie|kids)?\\.com|
                (?:www\\.)?deturl\\.com/www\\.youtube\\.com|
                (?:www\\.)?pwnyoutube\\.com|
                (?:www\\.)?hooktube\\.com|
                (?:www\\.)?yourepeat\\.com|
                tube\\.majestyc\\.net|
                youtube\\.googleapis\\.com)/
                (?:.*?\\#/)?
                (?:
                    (?:(?:v|embed|e|shorts|live)/(?!videoseries|live_stream))|
                    (?:
                        (?:(?:watch|movie)(?:_popup)?(?:\\.php)?/?)?
                        (?:\\?|\\#!?)
                        (?:.*?[&;])?
                        v=
                    )
                )
            )
            |(?:
                youtu\\.be|
                vid\\.plus|
                zwearz\\.com/watch
            )/
            |(?:www\\.)?cleanvideosearch\\.com/media/action/yt/watch\\?videoId=
        )
    )?
    ([0-9A-Za-z_-]{11})
    (?:.+)?
    (?:\\#|$)`.replace(/\s+/g, ""),
  "i",
);

export function extractVideoId(url: string): string | null {
  const match = url.match(YOUTUBE_URL_REGEX);
  return match ? match[1] : null;
}

export function isValidYouTubeUrl(url: string): boolean {
  return YOUTUBE_URL_REGEX.test(url);
}
