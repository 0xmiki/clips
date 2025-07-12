<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";
  import { Button } from "./ui/button/index";
  import Plus from '@lucide/svelte/icons/plus';
  import Minus from '@lucide/svelte/icons/minus';
  import RotateCw from '@lucide/svelte/icons/rotate-cw';
  import Clock from '@lucide/svelte/icons/clock';

  const props = $props<{
    videoUrl: string;
    startTime?: string;
    endTime?: string;
    width?: string;
    height?: string;
    controls?: boolean;
    autoplay?: boolean;
    muted?: boolean;
    showClipControls?: boolean;
    thumbnail?: string;
    streamingUrl: string;
    loading?: boolean;
  }>();

  const {
    videoUrl,
    startTime = "0:00",
    endTime = "",
    width = "100%",
    height = "auto",
    controls = true,
    autoplay = false,
    muted = false,
    showClipControls = false,
    thumbnail = "",
    streamingUrl: propStreamingUrl
  } = props;

  const loading = $derived(() => props.loading ?? false);

  let videoElement: HTMLVideoElement;
  let error = $state("");
  let initialAutoPlayTimeout: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (!propStreamingUrl) {
      error = "No streaming URL provided";
      return;
    }
    error = "";
  });

  const dispatch = createEventDispatcher();

  // Convert time string to seconds
  function timeToSeconds(timeStr: string): number {
    const parts = timeStr.split(":").map(Number).reverse();
    let seconds = 0;
    for (let i = 0; i < parts.length; i++) {
      seconds += parts[i] * Math.pow(60, i);
    }
    return seconds;
  }

  // Helper to format seconds as hh:mm:ss
  function formatTime(seconds: number): string {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);
    if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  // For plus/minus controls
  function adjustStart(delta: number) {
    const newStart = Math.max(0, timeToSeconds(startTime) + delta);
    dispatch('updateStart', { startTime: formatTime(newStart) });
  }
  function adjustEnd(delta: number) {
    const newEnd = Math.max(timeToSeconds(startTime) + 1, timeToSeconds(endTime) + delta);
    dispatch('updateEnd', { endTime: formatTime(newEnd) });
  }

  function setupClipBehavior() {
    if (!videoElement || !endTime) return;

    const startSeconds = timeToSeconds(startTime);
    const endSeconds = timeToSeconds(endTime);

    if (endSeconds <= startSeconds) return;

    // Set initial time
    videoElement.currentTime = startSeconds;

    // Listen for time updates to handle end of clip
    const timeUpdateHandler = () => {
      if (videoElement.currentTime >= endSeconds) {
        videoElement.pause();
        videoElement.currentTime = startSeconds;
      }
    };

    videoElement.addEventListener('timeupdate', timeUpdateHandler);

    // Listen for seeking to keep within bounds
    const seekingHandler = () => {
      if (videoElement.currentTime < startSeconds) {
        videoElement.currentTime = startSeconds;
      } else if (videoElement.currentTime > endSeconds) {
        videoElement.currentTime = endSeconds;
      }
    };

    videoElement.addEventListener('seeking', seekingHandler);

    // Cleanup function
    return () => {
      videoElement.removeEventListener('timeupdate', timeUpdateHandler);
      videoElement.removeEventListener('seeking', seekingHandler);
    };
  }

  function startFromBeginning() {
    if (videoElement) {
      videoElement.currentTime = timeToSeconds(startTime);
      videoElement.play();
      dispatch('startFromBeginning');
    }
  }

  // Add seekTo method
  export function seekTo(seconds: number) {
    if (videoElement) {
      videoElement.currentTime = seconds;
    }
  }

  onMount(() => {
    let cleanup: (() => void) | undefined;
    if (propStreamingUrl && videoElement) {
      cleanup = setupClipBehavior();
      // Auto-play, wait 1s, then pause
      videoElement.play().catch(() => {});
      if (initialAutoPlayTimeout) clearTimeout(initialAutoPlayTimeout);
      initialAutoPlayTimeout = setTimeout(() => {
        videoElement.pause();
      }, 1000);
    }
    return () => {
      if (cleanup) cleanup();
      if (initialAutoPlayTimeout) clearTimeout(initialAutoPlayTimeout);
    };
  });

  // Modify the streaming URL effect
  $effect(() => {
    if (propStreamingUrl && videoElement) {
      // Reset the video source
      videoElement.pause();
      videoElement.currentTime = 0;
      videoElement.src = propStreamingUrl;
      videoElement.load();

      const cleanup = setupClipBehavior();
      // Auto-play, wait 1s, then pause (on streamingUrl change)
      videoElement.play().catch(() => {});
      if (initialAutoPlayTimeout) clearTimeout(initialAutoPlayTimeout);
      initialAutoPlayTimeout = setTimeout(() => {
        videoElement.pause();
      }, 1000);
      return cleanup;
    }
  });
</script>

<div class="video-player-container" >
  {#if error}
    <div class="error-overlay">
      <div class="error-message">Error: {error}</div>
    </div>
  {:else}
    <div>
    <video
      bind:this={videoElement}
      controls={controls}
      autoplay={autoplay}
      muted={false}
      preload="metadata"
      class="aspect-video"
      style="width: 100%; height: 100%;"
      poster={thumbnail}
    >
      Your browser does not support the video tag.
    </video>
    {#if showClipControls}
    <div class="flex flex-col md:flex-row gap-3 bg-card rounded-lg shadow justify-center items-center p-3 mt-2 border ">
      <!-- <div class="flex items-center gap-2 text-foreground">
        <Clock class="w-4 h-4 mr-1 text-muted-foreground" />
        <span class="font-medium">Clip:</span>
        <span class="ml-1">{startTime} - {endTime}</span>
        <span class="ml-2 text-xs text-muted-foreground">({formatTime(timeToSeconds(endTime) - timeToSeconds(startTime))})</span>
      </div> -->
      <div class="flex flex-wrap gap-4 items-center justify-center">
        <div class="flex items-center gap-1">
          <span class="text-xs text-muted-foreground">Start</span>
          <Button size="icon" variant="ghost" onclick={() => adjustStart(-1)} aria-label="Decrease start">
            <Minus />
          </Button>
          <Button size="icon" variant="ghost" onclick={() => adjustStart(1)} aria-label="Increase start">
            <Plus />
          </Button>
          <span class="mx-2 font-mono text-sm">{startTime}</span>
        </div>
        <div class="flex items-center gap-1">
          <span class="text-xs text-muted-foreground">End</span>
          <Button size="icon" variant="ghost" onclick={() => adjustEnd(-1)} aria-label="Decrease end">
            <Minus />
          </Button>
          <Button size="icon" variant="ghost" onclick={() => adjustEnd(1)} aria-label="Increase end">
            <Plus />
          </Button>
          <span class="mx-2 font-mono text-sm">{endTime}</span>
        </div>
        <Button class="ml-2" variant="secondary" onclick={startFromBeginning} aria-label="Start from beginning">
          <RotateCw class="mr-2" /> Start from Beginning
        </Button>
      </div>
    </div>
  {/if}
</div>
  {/if}
</div>

<style>
  .video-player-container {
    position: relative;
    background: transparent;
    border-radius: 12px;
    overflow: hidden;
  }


  .loading-spinner {
    font-size: 16px;
    font-weight: 500;
  }

  .error-message {
    font-size: 14px;
    margin-bottom: 12px;
    text-align: center;
  }

  .retry-button {
    background: #3b82f6;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }

  .retry-button:hover {
    background: #2563eb;
  }

  video {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }

</style> 