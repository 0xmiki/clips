<script context="module" lang="ts">
  export interface DownloadItem {
    id: string; // unique id for each download
    title: string;
    duration?: string; // for clips
    status: 'downloading' | 'done' | 'error';
    error?: string;
    type: 'clip' | 'full';
  }
</script>
<script lang="ts">
  import Check from '@lucide/svelte/icons/check';
  import Loader2 from '@lucide/svelte/icons/loader-2';
  import X from '@lucide/svelte/icons/x';

  let {
    downloads = [],
    onClose,

  } = $props();

</script>

{#if downloads.length > 0}
  <div class="fixed bottom-4 left-4 z-50 flex flex-col gap-2">
    {#each downloads as download (download.id)}
      <div class="flex flex-row items-center gap-3 bg-[#232323] rounded-xl shadow-lg px-5 py-3 min-w-[260px] max-w-[350px]">
        {#if download.status === "done"}
        <span class="text-green-400 text-lg"><Check size={15}/></span>
        {:else if download.status === "downloading"}
        <Loader2 class="animate-spin" size={20} />
        {/if}
        <div class="flex flex-col flex-1 min-w-0">
          <div class="truncate font-semibold text-sm">{download.title}</div>
          {#if download.type === 'clip' && download.duration}
            <div class="text-xs text-gray-400">{download.duration}</div>
          {/if}
        </div>
        {#if download.status === 'downloading'}
        {:else if download.status === 'error'}
         Error <X size={15} class="text-red-400" />
        {/if}
        <button class="ml-2 text-gray-400 hover:text-white" onclick={() => onClose(download.id)} aria-label="Close"><X size={15}/></button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .animate-spin {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style> 