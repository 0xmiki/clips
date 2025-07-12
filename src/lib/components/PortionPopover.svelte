<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import { createEventDispatcher } from "svelte";
    import { fetchTranscript } from "$lib/transcript";
    import { cn } from "$lib/utils";

    const dispatch = createEventDispatcher();

    let { transcript, videoId = null, videoDuration = 0, startTime: initialStartTime = "00:00:00", endTime: initialEndTime = "00:00:00", isFullVideo: initialIsFullVideo = true, disabled = true, } = $props<{
        videoDuration?: number;
        startTime?: string;
        endTime?: string;
        isFullVideo?: boolean;
        disabled?: boolean;
        transcript: string;
        videoId: string | null;
    }>();

    let startTime = $state(initialStartTime);
    let endTime = $state(initialEndTime);
    let isFullVideo = $state(initialIsFullVideo);
    let loadingTranscript = $state(false);
    let transcriptError = $state("");

    // Convert seconds to HH:MM:SS format
    function formatTime(seconds: number): string {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }

    // Convert HH:MM:SS to seconds
    function timeToSeconds(time: string): number {
        const parts = time.split(":").map(Number).reverse();
        let seconds = 0;
        for (let i = 0; i < parts.length; i++) {
            seconds += parts[i] * Math.pow(60, i);
        }
        return seconds;
    }

    // Initialize with full video duration when videoDuration changes
    $effect(() => {
        if (videoDuration > 0) {
            endTime = formatTime(videoDuration);
            startTime = "00:00:00";
            isFullVideo = true;
        }
    });

    function handleStartTimeChange() {
        updateContext();
    }

    function handleEndTimeChange() {
        updateContext();
    }

    function updateContext() {
        const startSeconds = timeToSeconds(startTime);
        const endSeconds = timeToSeconds(endTime);
        
        // Check if this represents the full video
        isFullVideo = startSeconds === 0 && endSeconds >= videoDuration;
        
        dispatch('contextChange', {
            startTime,
            endTime,
            startSeconds,
            endSeconds,
            isFullVideo
        });
    }

    function getDisplayText(): string {
        if (isFullVideo) {
            return "Context";
        }
        return `${startTime} - ${endTime}`;
    }

    async function handleGetTranscript() {
        if (!videoId) return;
        loadingTranscript = true;
        transcriptError = "";
        try {
            const t = await fetchTranscript(videoId);
            transcript = t;
            // Save to localStorage
            localStorage.setItem(`transcript_${videoId}_en`, t);
            dispatch('transcriptFetched', { transcript: t });
        } catch (e) {
            transcriptError = e instanceof Error ? e.message : String(e);
        } finally {
            loadingTranscript = false;
        }
    }

    $effect(() => {
        if (videoId && transcript == null) {
            const t = localStorage.getItem(`transcript_${videoId}_en`);
            if (t) {
                transcript = t;
            }
        }
    });
</script>
    
<Popover.Root>
    <Popover.Trigger disabled={disabled}>
        <span class={cn(
            "text-xs text-white/50 flex items-center gap-1 px-2 py-0.5 border rounded-full",
            disabled ? "opacity-50 cursor-not-allowed" : "cursor-pointer hover:bg-white/5",
            videoId && !transcript ? "border-red-400/80" : "border-white/5"
        )}>
            {getDisplayText()}
        </span> 
    </Popover.Trigger>
    <Popover.Content class="w-80">
        <div class="grid gap-4">
            {#if !transcript}
                <div class="flex flex-col items-center gap-2 p-4">
                    <button
                        class="rounded-full bg-white/70 hover:bg-white/80 text-black/90 px-4 py-2 text-xs  disabled:opacity-50"
                        onclick={handleGetTranscript}
                        disabled={loadingTranscript || !videoId}
                    >
                        {#if loadingTranscript}
                            Loading...
                        {:else}
                            Get Transcript
                        {/if}
                    </button>
                    {#if transcriptError}
                        <div class="text-xs text-red-400 mt-2">{transcriptError}</div>
                    {/if}
                </div>
            {:else}
                <div class="space-y-2">
                    <h4 class="font-medium leading-none">Clip Context</h4>
                    <p class="text-muted-foreground text-sm">
                        Context to be used when creating clips
                    </p>
                </div>
                <div class="grid gap-2">
                    <div class="grid grid-cols-3 items-center gap-4">
                        <Label for="startTime">Start Time</Label>
                        <Input 
                            id="startTime" 
                            bind:value={startTime} 
                            oninput={handleStartTimeChange}
                            class="col-span-2 h-8" 
                            placeholder="00:00:00"
                        />
                    </div>
                    <div class="grid grid-cols-3 items-center gap-4">
                        <Label for="endTime">End Time</Label>
                        <Input 
                            id="endTime" 
                            bind:value={endTime} 
                            oninput={handleEndTimeChange}
                            class="col-span-2 h-8" 
                            placeholder="00:00:00"
                        />
                    </div>
                </div>
            {/if}
        </div>
    </Popover.Content>
</Popover.Root>