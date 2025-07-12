<script lang="ts">

    
    import Trash from '@lucide/svelte/icons/trash'

    interface HistoryItem {
        videoId: string;
        url: string;
        title: string;
        thumbnail: string;
        lastAccessed: number;
    }

    interface Props {
        show: boolean;
        videoHistory: HistoryItem[];
        onClose: () => void;
        onClearHistory: () => void;
        onRemoveFromHistory: (videoId: string) => void;
        onSelectVideo: (url: string) => void;
    }

    let { show, videoHistory, onClose, onClearHistory, onRemoveFromHistory, onSelectVideo }: Props = $props();
</script>

{#if show}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" onclick={onClose}>
        <div 
            class="w-[600px] rounded-[12px] border border-gray-500/30 bg-white p-6 dark:border-none dark:bg-[#303030] "
        >
            <div class="mb-4 flex items-center justify-between">
                <div class="flex items-center gap-4">
                    <!-- <h2 class="text-xl font-bold">History</h2> -->
                    {#if videoHistory.length > 0}
                        <button
                            onclick={() => {
                                    onClearHistory()
                            }}
                            class="text-sm text-white/50 hover:text-red-400/50"
                        >
                            Clear All
                        </button>
                    {/if}
                </div>
            </div>
            <div class="history-modal-content">

            
            {#if videoHistory.length === 0}
                <div class="flex flex-col items-center justify-center py-8 text-white/50">
                    <p>No history yet</p>
                </div>
            {:else}
                <div class="space-y-1 pr-7">
                    {#each videoHistory as item}
                        <div class="flex items-start opacity-60 hover:opacity-100 gap-2 rounded-[10px] transition-colors group ">
                            <div 
                                class="flex items-start gap-3 flex-1 cursor-pointer py-1 px-2 min-w-0"
                                onclick={() => {
                                    onSelectVideo(item.url);
                                    onClose();
                                }}
                            >
                                <img 
                                    src={item.thumbnail} 
                                    alt={item.title}
                                    class="h-13 w-23  rounded object-cover flex-shrink-0"
                                />
                                <div class="flex-1 min-w-0">
                                    <div class="truncate font-medium text-xs text-white/90">{item.title}</div>
                                    <div class="text-xs text-white/50">
                                        {new Date(item.lastAccessed).toLocaleDateString()}
                                    </div>
                                </div>
                            </div>
                            <button
                                onclick={() => {onRemoveFromHistory(item.videoId)}}
                                class="rounded p-2 text-white/50 hover:text-red-500/50 ml-1"
                                tabindex="0"
                            >
                                <Trash size={15} />
                            </button>
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
        </div>
    </div>
{/if}

<style>
    .history-modal-content {
        max-height: 44vh;
        overflow-y: auto;
    }
</style> 