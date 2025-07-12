<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import Loader2 from '@lucide/svelte/icons/loader-2';
    import Checkbox from './ui/checkbox/checkbox.svelte';

    interface Props {
        show: boolean;
        videoDuration: number;
        onClose: () => void;
        onCreateClip: (startTime: string, endTime: string, prompt: string, findMode: boolean) => Promise<void>;
    }

    let { show, videoDuration, onClose, onCreateClip }: Props = $props();

    let startTime = $state("00:00:00");
    let endTime = $state("00:00:00");
    let prompt = $state("");
    let creating = $state(false);
    let findMode = $state(false);

    // Time utility functions (copied from main page)
    function formatTime(seconds: number): string {
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
    }

    function timeToSeconds(ts: string): number {
        const parts = ts.split(":").map(Number).reverse();
        let seconds = 0;
        for (let i = 0; i < parts.length; i++) {
            seconds += parts[i] * Math.pow(60, i);
        }
        return seconds;
    }

    // Initialize with default values when modal opens
    $effect(() => {
        if (show && videoDuration > 0) {
            startTime = "00:00:00";
            endTime = formatTime(Math.min(120, videoDuration));
            findMode = false;
        }
    });

    $effect(() => {
        if (findMode && videoDuration > 0) {
            startTime = "00:00:00";
            endTime = formatTime(videoDuration);
        }
    });

    async function handleCreateClip() {
        if (creating) return;
        if (findMode && !prompt.trim()) {
            // Don't allow submission if prompt is empty in Find Mode
            return;
        }
        if (!findMode) {
            // Validate times
            const startSeconds = timeToSeconds(startTime);
            const endSeconds = timeToSeconds(endTime);
            if (startSeconds >= endSeconds) {
                alert("End time must be after start time");
                return;
            }
            if (endSeconds > videoDuration) {
                alert("End time cannot exceed video duration");
                return;
            }
        }
        creating = true;
        try {
            await onCreateClip(startTime, endTime, prompt, findMode);
            // Reset form
            startTime = "00:00:00";
            endTime = formatTime(Math.min(120, videoDuration));
            prompt = "";
            findMode = false;
            onClose();
        } catch (error) {
            console.error("Failed to create clip:", error);
            alert("Failed to create clip. Please try again.");
        } finally {
            creating = false;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleCreateClip();
        }
    }
</script>

{#if show}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
        <div 
            class="w-[500px] rounded-[12px] border border-gray-500/30 bg-white p-6 dark:border-none dark:bg-[#303030] relative"
            onclick={(e) => e.stopPropagation()}
        >
            {#if creating}
                <!-- Loading Overlay -->
                <div class="absolute inset-0 bg-[#303030]/90 rounded-[12px] flex items-center justify-center z-10">
                    <div class="flex flex-col items-center gap-3">
                        <Loader2 class="animate-spin" size={32} />
                        <p class="text-white/80 text-sm">Creating clip...</p>
                    </div>
                </div>
            {/if}
            
            <div class="mb-4">
                <h2 class="text-xl font-bold text-white">Add Custom Clip</h2>
                <div class="flex items-start gap-2 mt-2">
                    <Checkbox
                        id="findMode"
                        bind:checked={findMode}
                        disabled={creating}
                        class="data-[state=checked]:border-white/80 data-[state=checked]:bg-white/80 data-[state=checked]:text-black/80 mt-1 "
                    />
                    <div class="flex flex-col">
                        <label for="findMode" class="text-white/80 text-sm cursor-pointer">Find Mode</label>
                        <label for="findMode" class="text-white/80 text-xs cursor-pointer">Let the AI find a specific clip for you from the video</label>
                    </div>
                </div>
             
                    <!-- <div class="text-xs text-white/60 mt-1 mb-1"></div> -->
               
            </div>
            
            <div class="space-y-4">
                <!-- Time inputs -->
                <div class="flex gap-4">
                    <div class="flex-1">
                        <label class="block text-sm font-medium text-white/70 mb-2">Start Time</label>
                        <input
                            type="text"
                            bind:value={startTime}
                            class="w-full rounded-lg bg-[#404040] p-3 text-white border-none focus:ring-2 focus:ring-white/20 focus:outline-none"
                            placeholder="00:00:00"
                            disabled={creating || findMode}
                        />
                    </div>
                    <div class="flex-1">
                        <label class="block text-sm font-medium text-white/70 mb-2">End Time</label>
                        <input
                            type="text"
                            bind:value={endTime}
                            class="w-full rounded-lg bg-[#404040] p-3 text-white border-none focus:ring-2 focus:ring-white/20 focus:outline-none"
                            placeholder="00:00:00"
                            disabled={creating || findMode}
                        />
                    </div>
                </div>
                
                <!-- Prompt input -->
                <div>
                    <label class="block text-sm font-medium text-white/70 mb-2">
                        Prompt {#if findMode}<span class="text-red-400/70 text-xs">(Prompt is required in find mode)</span>{/if}
                    </label>
                    <textarea
                        bind:value={prompt}
                        onkeydown={handleKeydown}
                        rows={3}
                        class="w-full rounded-lg bg-[#404040] p-3 text-white border-none focus:ring-2 focus:ring-white/20 focus:outline-none resize-none"
                        placeholder={findMode ? "Describe what you want the AI to find (required)" : "What kind of clip do you want? (Optional)"}
                        disabled={creating}
                    />

                </div>
                
                <!-- Buttons -->
                <div class="flex gap-3 justify-end ">
                    <!-- <button
                        onclick={onClose}
                        class="px-4 py-2 rounded-lg text-white/70 hover:text-white transition-colors"
                        disabled={creating}
                    >
                        Cancel
                    </button> -->
                    <button
                        onclick={handleCreateClip}
                        disabled={creating || (findMode && !prompt.trim())}
                        class="rounded-[15px] bg-white/80 py-2 w-[90px] text-xs disabled:opacity-40 dark:text-black"
                    >
                        Create Clip
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if} 