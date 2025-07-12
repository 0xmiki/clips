<script lang="ts">

    import { open as openDialog } from'@tauri-apps/plugin-dialog';
    import Checkbox from './ui/checkbox/checkbox.svelte';

    interface Props {
        show: boolean;
        apiKey: string;
        downloadDir: string;
        model: string;
        showBgImage: boolean;
        onClose: () => void;
        onSave: () => void;
        onApiKeyChange: (value: string) => void;
        onDownloadDirChange: (value: string) => void;
        onModelChange: (value: string) => void;
        onShowBgImageChange: (value: boolean) => void;
    }

    let { show, apiKey, downloadDir, model, showBgImage, onClose, onSave, onApiKeyChange, onDownloadDirChange, onModelChange, onShowBgImageChange }: Props = $props();

 
    const modelOptions = [
        { name: "Gemini 2.5 Pro", value: "gemini-2.5-pro" },
        { name: "Gemini 2.5 Flash", value: "gemini-2.5-flash" },
        { name: "Gemini 2.5 Flash-Lite Preview", value: "gemini-2.5-flash-lite-preview-06-17" },
        { name: "Gemini 2.0 Flash", value: "gemini-2.0-flash" },
        { name: "Gemini 2.0 Flash-Lite", value: "gemini-2.0-flash-lite" }
    ];


    async function saveAndClose() {
        await onSave();
        onClose();
    }
</script>

{#if show}
    <div class="fixed inset-0  z-50 flex items-center justify-center">

        <div class="absolute inset-0 bg-black/50" onclick={saveAndClose}></div>

        <div 
            class="relative pb-7 w-[500px] rounded-[12px] border border-gray-500/30 bg-white p-6 dark:border-none dark:bg-[#303030]"
        >
            <div class="space-y-3">
                <div class="">
                    <p class="text-xs font-medium text-white/70 mb-1">Gemini API Key</p>
                    <input
                        type="password"
                        value={apiKey}
                        oninput={(e) => onApiKeyChange(e.currentTarget.value)}
                        class="w-full rounded-lg bg-[#404040] p-2 text-sm"
                        placeholder="Enter your Gemini API key"
                    />
                </div>

       
                <div class="">
                    <p class="text-xs mb-1 font-medium text-white/70">AI Model</p>
                    <select
                        value={model}
                        onchange={(e) => onModelChange(e.currentTarget.value)}
                        class="w-full rounded-lg bg-[#404040] p-2 text-sm text-white/70"
                    >
                        {#each modelOptions as option}
                            <option value={option.value}>{option.name}</option>
                        {/each}
                    </select>
                </div>

                <div class="">
                    <p class="text-xs mb-1 font-medium text-white/70">Download Directory</p>
                    <div class="flex gap-2 items-center">
                        <input
                            type="text"
                            value={downloadDir}
                            readonly
                            class="w-full rounded-lg bg-[#404040] text-white/70 p-2 text-sm cursor-pointer opacity-80"
                            placeholder="Select download directory"
                            onclick={async () => {
                                const selected = await openDialog({ directory: true, multiple: false });
                                if (selected) {
                                    onDownloadDirChange(selected as string);
                                }
                            }}
                        />
                    </div>
                </div>
                <div class="flex ml-1 items-center mt-2">
                    <Checkbox 
                    id="show-bg-image"
                        checked={showBgImage}
                        onCheckedChange={(e) => onShowBgImageChange(e)}
                        class="mr-2"
                    />
                    <label for="show-bg-image" class="text-xs font-medium text-white/70">Show Background Image</label>
                </div>
            </div>
        </div>
    </div>
{/if} 