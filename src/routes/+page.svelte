<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { readText } from '@tauri-apps/plugin-clipboard-manager';
    import { confirm } from '@tauri-apps/plugin-dialog';
    import VideoPlayer from "$lib/components/VideoPlayer.svelte";
    import HistoryModal from "$lib/components/HistoryModal.svelte";
    import SettingsModal from "$lib/components/SettingsModal.svelte";
    import { GoogleGenAI } from "@google/genai";
    import Copy from '@lucide/svelte/icons/copy';
    import CopyCheck from '@lucide/svelte/icons/check';
    import Loader2 from '@lucide/svelte/icons/loader-2';
    import Download from '@lucide/svelte/icons/download';
    import X from '@lucide/svelte/icons/x';
    import Settings from '@lucide/svelte/icons/settings';
    import Clock from '@lucide/svelte/icons/clock';
    import RefreshCw from '@lucide/svelte/icons/refresh-cw';
    import { fetchTranscript } from '$lib/transcript';
    import DownloadProgressModal from "$lib/components/DownloadProgressModal.svelte";
    import type { DownloadItem } from '$lib/components/DownloadProgressModal.svelte';
    import PortionPopover from "$lib/components/PortionPopover.svelte";
    import AddClipModal from "$lib/components/AddClipModal.svelte";
    import {truncateTranscript, formatTime, timeToSeconds, extractVideoId, isValidYouTubeUrl, sys_ins_agent_1, sys_ins_agent_2, sys_ins_agent_custom_clip } from "$lib/functions/index"

    type Format = {
        format_id: string;
        ext: string;
        quality: string;
        filesize?: number;
        resolution: string;
    };

    let downloads: DownloadItem[] = $state([]);

    function removeDownload(id: string) {
      downloads = downloads.filter(d => d.id !== id);
    }
  
    // Settings state
    let showSettingsModal = $state(false);
    let GEMINI_API_KEY = $state(localStorage.getItem('gemini_api_key') || "");
    const genAI = $derived(new GoogleGenAI({ apiKey: GEMINI_API_KEY ?? "" }));
    let downloadDir = $state(localStorage.getItem('download_directory') || "~/Downloads");
    let model = $state(localStorage.getItem('gemini_model') || "gemini-2.0-flash");
    let selectedFormat = $state<Format | null>(null);
    let selectedFullVideoFormat = $state<Format | null>(null);
    let downloading = $state(false);
    let downloadError = $state<string | null>(null);
    let downloadingFullVideo = $state(false);
    let fullVideoDownloadError = $state<string | null>(null);
    let numClips = $state(5);
    let maxClipDuration = $state(3);

    // Context state for transcript truncation
    let contextStartTime = $state("00:00:00");
    let contextEndTime = $state("00:00:00");
    let contextStartSeconds = $state(0);
    let contextEndSeconds = $state(0);
    let isFullVideoContext = $state(true);

    let transcript = $state<string | null>(null);


    // Save settings to localStorage
    function saveSettings() {
        localStorage.setItem('gemini_api_key', GEMINI_API_KEY);
        localStorage.setItem('download_directory', downloadDir);
        localStorage.setItem('gemini_model', model);
        localStorage.setItem('show_bg_image', String(showBgImage));
        showSettingsModal = false;
    }

    let video = $state<{
        url: string;
        title: string;
        thumbnail: string;
        author: string;
        duration: number;
        transcript: string | null;
        formats: Format[];
    } | null>(null);

    let loading = $state(false);
    let streamingUrl = $state('');
    let urlInput = $state('');
    let copied = $state(false);

    let promptInput = $state("");
    let generatedClips = $state<any[]>([]);
    let generating = $state(false);
    let selectedClipIndex = $state<number>(-1);
    let videoPlayer: any;

    let streamingUrlLoading = $state(false);

    let videoHistory = $state(getHistory());
    let showHistoryModal = $state(false);
    let showAddClipModal = $state(false);
    // Add request cancellation system to prevent race conditions
    // Each processVideo() call gets a unique requestId. Only the most recent request
    // can update the state, preventing race conditions when users quickly switch videos.
    let currentRequestId = $state(0);

    let filteredFormats = $derived(video?.formats?.filter(format => 
        format.quality !== "storyboard" && format.quality !== "unknown"
    ) || []);

    function getVideoFromCache(url: string) {
        try {
            const videoId = extractVideoId(url);
            if (!videoId) return null;
            
            const cachedData = localStorage.getItem(`video_${videoId}`);
            if (!cachedData) return null;
            
            const { data } = JSON.parse(cachedData);
            return data;
        } catch (e) {
            console.error("Failed to get video from cache", e);
            return null;
        }
    }

    function saveVideoToCache(url: string, videoData: any) {
        try {
            const videoId = extractVideoId(url);
            if (!videoId) return;
            
            localStorage.setItem(`video_${videoId}`, JSON.stringify({
                data: videoData,
                clips: [],  
                lastUpdated: Date.now()
            }));
            
            addToHistory(videoId, url, videoData.title, videoData.thumbnail);
        } catch (e) {
            console.error("Failed to save video to cache", e);
        }
    }

    function getStreamingUrlFromCache(url: string) {
        try {
            const videoId = extractVideoId(url);
            if (!videoId) return null;
            
            const cachedData = localStorage.getItem(`stream_${videoId}`);
            if (!cachedData) return null;
            
            const { streamingUrl, expiry } = JSON.parse(cachedData);
            if (Date.now() > expiry) {
                localStorage.removeItem(`stream_${videoId}`);
                return null;
            }
            
            return streamingUrl;
        } catch (e) {
            console.error("Failed to get streaming URL from cache", e);
            return null;
        }
    }

    function saveStreamingUrlToCache(url: string, streamingUrl: string) {
        try {
            const videoId = extractVideoId(url);
            if (!videoId) return;
            
            const expiry = Date.now() + (6 * 60 * 60 * 1000); // 6 hours from now
            localStorage.setItem(`stream_${videoId}`, JSON.stringify({
                streamingUrl,
                expiry
            }));
        } catch (e) {
            console.error("Failed to save streaming URL to cache", e);
        }
    }

    function getClipsFromCache(url: string) {
        try {
            const videoId = extractVideoId(url);
            if (!videoId) return null;
            
            const cachedData = localStorage.getItem(`video_${videoId}`);
            if (!cachedData) return null;
            
            const { clips } = JSON.parse(cachedData);
            return clips || [];
        } catch (e) {
            console.error("Failed to get clips from cache", e);
            return null;
        }
    }

    function saveClipsToCache(url: string, clips: any[]) {
        try {
            const videoId = extractVideoId(url);
            if (!videoId) return;
            
            const cachedData = localStorage.getItem(`video_${videoId}`);
            if (!cachedData) return;
            
            const data = JSON.parse(cachedData);
            data.clips = clips;
            data.lastUpdated = Date.now();
            
            localStorage.setItem(`video_${videoId}`, JSON.stringify(data));
        } catch (e) {
            console.error("Failed to save clips to cache", e);
        }
    }

    function addToHistory(videoId: string, url: string, title: string, thumbnail: string) {
        try {
            const history = JSON.parse(localStorage.getItem('video_history') || '[]');
            const existingIndex = history.findIndex((item: any) => item.videoId === videoId);
            
            const historyItem = {
                videoId,
                url,
                title,
                thumbnail,
                lastAccessed: Date.now()
            };
            
            if (existingIndex !== -1) {
                history.splice(existingIndex, 1);
            }
            
            history.unshift(historyItem);
            
            // Keep only last 50 items
            if (history.length > 50) {
                history.pop();
            }
            
            localStorage.setItem('video_history', JSON.stringify(history));
            videoHistory = history;
        } catch (e) {
            console.error("Failed to add to history", e);
        }
    }

    function getHistory() {
        try {
            return JSON.parse(localStorage.getItem('video_history') || '[]');
        } catch (e) {
            console.error("Failed to get history", e);
            return [];
        }
    }

    function clearHistory() {
        // Remove all video_ and stream_ keys from localStorage
        for (let i = localStorage.length - 1; i >= 0; i--) {
            const key = localStorage.key(i);
            if (key && (key.startsWith('video_') || key.startsWith('stream_'))) {
                localStorage.removeItem(key);
            }
        }
        localStorage.setItem('video_history', '[]');
        videoHistory = [];
    }

    function removeFromHistory(videoId: string) {
        try {
            const history = getHistory();
            const filteredHistory = history.filter((item: any) => item.videoId !== videoId);
            localStorage.setItem('video_history', JSON.stringify(filteredHistory));
            videoHistory = filteredHistory;

            // Remove associated caches
            localStorage.removeItem(`video_${videoId}`);
            localStorage.removeItem(`stream_${videoId}`);
        } catch (e) {
            console.error("Failed to remove from history", e);
        }
    }


    async function getStreamingUrl(requestId?: number) {
        if (!video?.url) return;
        const currentId = requestId ?? currentRequestId;
        
        streamingUrlLoading = true;
        try {
            if (currentId !== currentRequestId) return;
    
            const cachedUrl = getStreamingUrlFromCache(video.url);
            if (cachedUrl) {
                
                if (currentId !== currentRequestId) return;
                streamingUrl = cachedUrl;
                return;
            }
            
            const result = await invoke("get_streaming_url", { videoUrl: video.url }) as { streaming_url: string };
            
            
            if (currentId !== currentRequestId) return;
            
            streamingUrl = result.streaming_url;
            saveStreamingUrlToCache(video.url, streamingUrl);
        } catch (e) {
           
            if (currentId === currentRequestId) {
                console.error("Failed to get streaming URL", e);
            }
        } finally {
            
            if (currentId === currentRequestId) {
                streamingUrlLoading = false;
            }
        }
    }

   
    function resetVideoState() {
        video = null;
        generatedClips = [];
        selectedClipIndex = -1;
        streamingUrl = '';
    }

    async function processVideo() {
        
        const requestId = ++currentRequestId;
        
        resetVideoState(); 
        if (!urlInput.trim()) return;
        
        if (!isValidYouTubeUrl(urlInput)) {
            alert("Please enter a valid YouTube URL");
            return;
        }
        
        loading = true;
        try {
            if (requestId !== currentRequestId) return;
           
            const cachedData = getVideoFromCache(urlInput);
            if (cachedData) {
                if (requestId !== currentRequestId) return;
                video = cachedData;
                console.log("Video loaded from cache:", video);
                initializeContext();
                streamingUrl = '';
                await getStreamingUrl(requestId);
                if (requestId !== currentRequestId) return;
                const cachedClips = getClipsFromCache(urlInput);
                if (cachedClips) {
                    generatedClips = cachedClips;
                    if (generatedClips.length > 0) {
                        selectClip(0);
                    }
                } else {
                    generatedClips = [];
                    selectedClipIndex = -1;
                }
                loading = false;
                return;
            }
            if (requestId !== currentRequestId) return;
            // Parallelize metadata and transcript fetch
            const videoId = extractVideoId(urlInput);
            const metadataPromise = invoke("extract_video_metadata", { url: urlInput }) as Promise<{
                url: string;
                title: string;
                thumbnail: string;
                author: string;
                duration: number;
                formats: Format[];
            }>;
            const transcriptPromise = videoId ? fetchTranscript(videoId) : Promise.resolve(null);
            const [videoData, transcript] = await Promise.all([metadataPromise, transcriptPromise]);
            if (requestId !== currentRequestId) return;
            if (!videoData) {
                throw new Error("Failed to extract video metadata");
            }
            video = {
                url: videoData.url,
                title: videoData.title,
                thumbnail: videoData.thumbnail,
                author: videoData.author,
                duration: videoData.duration,
                formats: videoData.formats,
                transcript: transcript || null
            };
            initializeContext();
            saveVideoToCache(urlInput, video);
            streamingUrl = '';
            await getStreamingUrl(requestId);
            if (requestId !== currentRequestId) return;
            generatedClips = [];
            selectedClipIndex = -1;
        } catch (e) {
            if (requestId === currentRequestId) {
                console.error("Failed to process video", e);
                alert("Failed to process video. Please check the URL and try again." + e);
            }
        } finally {
            if (requestId === currentRequestId) {
                loading = false;
            }
        }
    }

    async function generateClips() {
        if (!video) return;


        if (!GEMINI_API_KEY || GEMINI_API_KEY.trim() === '') {
            const confirmation = await confirm(
                'Please set your Gemini API key in settings to generate clips.',
                { title: 'API Key Required', kind: 'warning' }
            );
            const result = await confirmation

            // if (result === "Open Settings") {
            //     showSettingsModal = true;
            // }
            return;
        }
        
        generating = true;
        try {
            const transcript = video.transcript ?? "";
            if (!transcript) {
                alert("No transcript available for this video. Please try a different video or check if the video has captions.");
                generating = false;
                return;
            }
            
            // Truncate transcript based on context if not full video
            let processedTranscript = transcript;
            if (!isFullVideoContext && contextStartSeconds !== contextEndSeconds) {
                processedTranscript = truncateTranscript(transcript, contextStartSeconds, contextEndSeconds);
                console.log("Using truncated transcript from", contextStartTime, "to", contextEndTime);
                console.log(processedTranscript)
            } else {
                console.log("Using full video transcript");
            }
            
            const prompt = sys_ins_agent_1(promptInput, maxClipDuration, numClips, processedTranscript)

            let config = { responseMimeType: 'application/json' };
            let contents = [{ role: 'user', parts: [{ text: prompt }] }];
            const result = await genAI.models.generateContent({ model, config, contents });
            const text = result.text;
            if (!text) {
                alert('AI did not return any text.');
                generating = false;
                return;
            }

            const improveCaptionsPrompt = sys_ins_agent_2(text)
            const improveResult = await genAI.models.generateContent({
                model,
                config,
                contents: [{ role: 'user', parts: [{ text: improveCaptionsPrompt }] }]
            });
            const improvedText = improveResult.text;
            if (!improvedText) {
                alert('Caption improver did not return any text.');
                generating = false;
                return;
            }
            try {
                generatedClips = JSON.parse(improvedText ?? "[]");
                if (video.url) {
                    saveClipsToCache(video.url, generatedClips);
                }
                if (generatedClips.length > 0) {
                    selectClip(0);
                }
            } catch (e) {
                console.error("Failed to parse improved AI JSON response", improvedText);
                alert("Failed to parse improved AI JSON response");
            }
        } catch (e) {
            console.error("generation error", e);
            alert("Error generating clips");
        }
        generating = false;
    }

    function selectClip(index: number) {
        selectedClipIndex = index;
        if (index >= 0 && index < generatedClips.length) {
            const clip = generatedClips[index];
            const startTime = timeToSeconds(clip.start_time);
            if (videoPlayer) {
                videoPlayer.seekTo(startTime);
            }
        }
    }

    async function handlePasteClick() {
        try {
            const text = await readText();
            if (text) {
                if (!isValidYouTubeUrl(text)) {
                    alert("The clipboard content is not a valid YouTube URL");
                    return;
                }
                resetVideoState(); 
                urlInput = text;
                copied = true;
                setTimeout(() => {
                    copied = false;
                }, 2000);
                if (text.trim()) {
                    processVideo();
                }
            }
        } catch (err) {
            console.error('Failed to read clipboard:', err);
        }
    }

    function addDownload(item: DownloadItem) {
      downloads = [...downloads, item];
    }
    function updateDownload(id: string, update: Partial<DownloadItem>) {
      downloads = downloads.map(d => d.id === id ? { ...d, ...update } : d);
    }

    async function downloadClip(clip: any) {
        if (!video || !selectedFormat) return;
        const id = `${video.url}_${clip.start_time}_${clip.end_time}_${selectedFormat.format_id}_${Date.now()}`;
        addDownload({
            id,
            title: video.title,
            duration: `${clip.start_time} - ${clip.end_time}`,
            status: 'downloading',
            type: 'clip',
        });
        try {
            const title = video.title.slice(0, 50).trim();
            const timestamp = `${clip.start_time.replace(':', '_')}-${clip.end_time.replace(':', '_')}`;
            const quality = selectedFormat.quality.toLowerCase().replace(' ', '');
            const filename = `${title}_${timestamp}_${quality}`;
            await invoke('download_clip', {
                videoUrl: video.url,
                formatId: selectedFormat.format_id,
                startTime: clip.start_time,
                endTime: clip.end_time,
                outputDir: downloadDir,
                filename
            });
            updateDownload(id, { status: 'done' });
        } catch (error) {
            updateDownload(id, { status: 'error', error: error instanceof Error ? error.message : String(error) });
        }
    }

    async function downloadFullVideo() {
        if (!video || !selectedFullVideoFormat) return;
        const id = `${video.url}_full_${selectedFullVideoFormat.format_id}_${Date.now()}`;
        addDownload({
            id,
            title: video.title,
            status: 'downloading',
            type: 'full',
        });
        try {
            const title = video.title.slice(0, 50).trim();
            const quality = selectedFullVideoFormat.quality.toLowerCase().replace(' ', '');
            const filename = `${title}_${quality}`;
            await invoke('download_full_video', {
                videoUrl: video.url,
                formatId: selectedFullVideoFormat.format_id,
                outputDir: downloadDir,
                filename
            });
            updateDownload(id, { status: 'done' });
        } catch (error) {
            updateDownload(id, { status: 'error', error: error instanceof Error ? error.message : String(error) });
        }
    }

    function handleContextChange(event: CustomEvent) {
        const { startTime, endTime, startSeconds, endSeconds, isFullVideo } = event.detail;
        contextStartTime = startTime;
        contextEndTime = endTime;
        contextStartSeconds = startSeconds;
        contextEndSeconds = endSeconds;
        isFullVideoContext = isFullVideo;
    }

    function initializeContext() {
        if (video?.duration) {
            contextEndTime = formatTime(video.duration);
            contextStartTime = "00:00:00";
            contextStartSeconds = 0;
            contextEndSeconds = video.duration;
            isFullVideoContext = true;
        }
    }

    async function refreshVideo() {
        if (!video?.url) return;
        
        streamingUrl = '';
        streamingUrlLoading = true;
        
        const requestId = ++currentRequestId;
        
        try {
            const videoId = extractVideoId(video.url);
            if (videoId) {
                localStorage.removeItem(`stream_${videoId}`);
            }
            await getStreamingUrl(requestId);
            if (requestId === currentRequestId) {
                console.log("Video stream refreshed successfully");
            }
        } catch (e) {
            console.error("Failed to refresh video", e);
        } finally {
            if (requestId === currentRequestId) {
                streamingUrlLoading = false;
            }
        }
    }

    async function generateCustomClip(startTime: string, endTime: string, customPrompt: string = "", findMode: boolean = false) {
        if (!video) return;
        
        // Check if API key is set
        if (!GEMINI_API_KEY || GEMINI_API_KEY.trim() === '') {
            const confirmation = await confirm(
                'Please set your Gemini API key in settings to generate clips.',
                { title: 'API Key Required', kind: 'warning' }
            );
            const result = await confirmation

            // if (result === "Open Settings") {
            //     showSettingsModal = true;
            // }
            return;
        }
        
        try {
            const transcript = video.transcript ?? "";
            if (!transcript) {
                alert("No transcript available for this video. Please try a different video or check if the video has captions.");
                return;
            }
            let processedTranscript = "";
            if (findMode) {
                processedTranscript = transcript;
            } else {
                const startSeconds = timeToSeconds(startTime);
                const endSeconds = timeToSeconds(endTime);
                processedTranscript = truncateTranscript(transcript, startSeconds, endSeconds);
            }
            // Use sys_ins_agent_custom_clip to build the prompt
            const prompt = sys_ins_agent_custom_clip(
                startTime,
                endTime,
                customPrompt,
                processedTranscript,
                findMode
            );

            let config = { responseMimeType: 'application/json' };
            let contents = [{ role: 'user', parts: [{ text: prompt }] }];
            const result = await genAI.models.generateContent({ model, config, contents });
            const text = result.text;
            if (!text) {
                alert('AI did not return any text.');
                return;
            }

            const improveCaptionsPrompt = `\nYou are an expert content editor. Your task is to take a list of video clip segments (each with start_time, end_time, quotes, and a caption) and rewrite the captions to maximize the value provided per word. Make each caption as information-dense, insightful, and clear as possible, focusing on delivering actionable takeaways, key lessons, or unique perspectives. Avoid personal, emotional, or conversational language. Do not use filler, hype, or generic phrasing. Do not change the quotes or timing, only improve the caption field for each segment. Return the improved list in the same JSON structure.\n\nHere is the list of clips:\n${text}\n`;
            const improveResult = await genAI.models.generateContent({
                model,
                config,
                contents: [{ role: 'user', parts: [{ text: improveCaptionsPrompt }] }]
            });
            const improvedText = improveResult.text;
            if (!improvedText) {
                alert('Caption improver did not return any text.');
                return;
            }
            
            try {
                const newClip = JSON.parse(improvedText ?? "[]")[0]; 
                if (newClip) {
                    generatedClips = [newClip, ...generatedClips];
                    if (video.url) {
                        saveClipsToCache(video.url, generatedClips);
                    }
                    selectClip(0);
                }
            } catch (e) {
                console.error("Failed to parse improved AI JSON response", improvedText);
                alert("Failed to parse improved AI JSON response");
            }
        } catch (e) {
            console.error("Custom clip generation error", e);
            alert("Error generating custom clip");
        }
    }

    function handleTranscriptFetched(event: CustomEvent) {
        transcript = event.detail.transcript;
        if (video) video.transcript = transcript;
    }

    $effect(() => {
        if (video && Array.isArray(video.formats) && video.formats.length > 0) {

            const filteredFormats = video.formats.filter(format => 
                format.quality !== "storyboard" && format.quality !== "unknown"
            );

            const parseResolution = (res: string) => {
                if (!res || res === "N/A") return 0;
                const [w, h] = res.split('x').map(Number);
                return (w || 0) * (h || 0);
            };
            const sortedFormats = filteredFormats.slice().sort((a, b) => {
                const resA = parseResolution(a.resolution);
                const resB = parseResolution(b.resolution);
                if (resA !== resB) return resB - resA;
                return (b.filesize || 0) - (a.filesize || 0);
            });

            const bestFormat = sortedFormats[0];
            if (bestFormat) {
                selectedFormat = bestFormat;
                selectedFullVideoFormat = bestFormat;
            }
        }
    });

    $effect(() => {
        if (video?.duration && isFullVideoContext) {
            contextEndTime = formatTime(video.duration);
            contextEndSeconds = video.duration;
        }
    });
    
    $effect(() => {
        if (video) {
            transcript = video.transcript ?? null;
        } else {
            transcript = null;
        }
    });

    let showBgImage = $state(localStorage.getItem('show_bg_image') === null ? true : localStorage.getItem('show_bg_image') === 'true');

    onMount(() => {
        const savedApiKey = localStorage.getItem('gemini_api_key');
        const savedDownloadDir = localStorage.getItem('download_directory');
        const savedModel = localStorage.getItem('gemini_model');
        const savedShowBgImage = localStorage.getItem('show_bg_image');
        if (savedApiKey) GEMINI_API_KEY = savedApiKey;
        if (savedDownloadDir) downloadDir = savedDownloadDir;
        if (savedModel) model = savedModel;
        if (savedShowBgImage !== null) showBgImage = savedShowBgImage === 'true';
    });

</script>

<div style="background-image: {showBgImage ? "url('/gray.png')" : 'none'}; background-size: cover; background-position: center; background-repeat: no-repeat; background-attachment:fixed" class="min-h-screen flex flex-col p-4">
    {#if video}
        <div class="flex w-full gap-6 px-2 md:px-4 lg:px-6 flex-col lg:flex-row mt-5">
            <!-- Video Section  -->
            <div class="w-full lg:w-[50%]">
                {#if video.thumbnail}
                    <div class="lg:sticky lg:top-4">
                        <div class="">
                            {#if streamingUrlLoading}
                                <div class="aspect-video flex items-center justify-center bg-[#212121]/90 rounded-lg" style="width: 100%;">
                                    <Loader2 class="animate-spin" size={48} />
                                </div>
                            {:else}
                                {#key video?.url + streamingUrl}
                                <VideoPlayer 
                                    bind:this={videoPlayer}
                                    videoUrl={video.url}
                                    streamingUrl={streamingUrl}
                                    thumbnail={video.thumbnail}
                                    width="100%" 
                                    height="100%"
                                    controls={true}
                                    autoplay={false}
                                    muted={true}
                                />
                                {/key}
                            {/if}
                        </div>
                        <!-- Add format selector and download button -->
                        <div class="flex mt-2 md:px-3 justify-between items-center">
                        <div class=" flex items-center gap-3 flex-row ">
                            <select 
                                bind:value={selectedFullVideoFormat} 
                                class="rounded-full bg-[#404040] p-2 px-4 h-10 text-sm w-[333px]"
                            >
                                {#if filteredFormats && filteredFormats.length > 0}
                                    {#each filteredFormats as format}
                                        <option value={format}>
                                            {format.quality} ({format.resolution}) - {format.ext}
                                            {#if format.filesize}
                                                ({(format.filesize / 1024 / 1024).toFixed(1)} MB)
                                            {/if}
                                        </option>
                                    {/each}
                                {:else}
                                    <option value={null}>No formats available</option>
                                {/if}
                            </select>
                            <button 
                                class="mr-2 flex items-center justify-center gap-2 rounded-full hover:bg-white/90 bg-white/50 px-2 py-2 font-medium text-black/80 disabled:opacity-50 h-10 aspect-square "
                                disabled={downloadingFullVideo || !selectedFullVideoFormat}
                                onclick={downloadFullVideo}
                            >
                                {#if downloadingFullVideo}
                                    <Loader2 class="animate-spin" size={16} />
                                    Downloading...
                                {:else if fullVideoDownloadError}
                                    <X size={16} class="text-red-500" />
                                    Failed to download
                                {:else}
                                    <Download size={16} />
                                   
                                {/if}
                            </button>
                        </div>
                        <button 
                                class="flex  items-center justify-center gap-2 rounded-full hover:bg-white/90 bg-white/50 px-2 py-2 font-medium text-black/80 disabled:opacity-50 h-10 aspect-square"
                                disabled={streamingUrlLoading}
                                onclick={refreshVideo}
                                title="Reload Video"
                            >
                                {#if streamingUrlLoading}
                                    <Loader2 class="animate-spin" size={16} />
                                {:else}
                                    <RefreshCw size={16} />
                                {/if}
                            </button>
                        </div>

                        {#if fullVideoDownloadError}
                            <div class="mt-2 text-sm text-red-500 bg-red-500/10 p-2 rounded">
                                {fullVideoDownloadError}
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>
            
            <!-- Generated Clips Section -->
            <div class="w-full lg:w-[50%] mt-6 lg:mt-0">
                <div class="rounded-lg h-full overflow-y-auto">
                    {#if generating}
                        <div class="flex flex-col items-center self-center justify-center h-[200px] text-white/80">
                            <Loader2 class="animate-spin mb-2" size={24} />
                            <p>Generating clips...</p>
                        </div>
                    {:else if generatedClips.length === 0}
                        <div class="flex flex-col items-center justify-center h-[200px] text-white/80">
                            <p>No clips generated</p>
                            <p class="text-sm mt-2">Press "Get Clips" to start</p>
                        </div>
                    {:else}
                        <!-- Clips Grid -->
                        <div class="w-full overflow-x-auto mb-6 pb-2">
                            <div class="flex gap-2  min-w-max">
                                <div class="self-center">
                                    <button
                                        onclick={() => (showAddClipModal = true)}
                                        class="rounded-full flex items-center justify-center aspect-square bg-white/60 hover:bg-white/80 py-2 h-10  text-xs disabled:opacity-40 dark:text-black"
                                        disabled={!video}
                                    >
                                        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path d="M5 12h14"/>
                                            <path d="M12 5v14"/>
                                        </svg>
                                    </button>
                                </div>
                                {#each generatedClips as clip, idx}
                                    <div 
                                        class="cursor-pointer rounded-xl p-4  w-[280px] sm:w-[300px] hover:bg-[#303030] {selectedClipIndex === idx ? 'bg-[#303030]' : 'bg-[#303030]/55'}"
                                        onclick={() => selectClip(idx)}
                                    >
                                        <p class="text-xs font-medium mb-2">{clip.caption}</p>
                                        <p class="text-xs text-white/60">{clip.start_time} - {clip.end_time}</p>
                                    </div>
                                {/each}
                            </div>
                        </div>
                        
                        <!-- Selected Clip Details -->
                        {#if selectedClipIndex >= 0 && selectedClipIndex < generatedClips.length}
                            {@const selectedClip = generatedClips[selectedClipIndex]}
                            <div class="bg-[#303030]/80 rounded-lg p-4 sm:p-6">
                                <!-- Duration Controls -->
                                <div class="flex items-center gap-4  mb-4 flex-row">
                                    <div class="flex items-center gap-2 ">
                                        <span class="text-sm font-medium">Start:</span>
                                        <input
                                            type="text"
                                            bind:value={selectedClip.start_time}
                                            class="text-sm font-mono px-2 py-1 border rounded w-25 text-center"
                                            placeholder="00:00"
                                        />
                                    </div>
                                    
                                    <div class="flex items-center gap-2 ">
                                        <span class="text-sm font-medium">End:</span>
                                        <input
                                            type="text"
                                            bind:value={selectedClip.end_time}
                                            class="text-sm font-mono px-2 py-1 border rounded w-25 text-center"
                                            placeholder="00:00"
                                        />
                                    </div>
                                </div>
                                
                                <!-- Quotes -->
                                <div class="space-y-2">
                                    <ul class="space-y-1">
                                        {#each selectedClip.quotes as quote}
                                            <li class="text-sm bg-background/80 p-2 rounded border-l-2 border-primary/20">
                                                "{quote}"
                                            </li>
                                        {/each}
                                    </ul>
                                </div>
                                <div class="flex flex-col mt-4 gap-2">
                                    <!-- Format Selection -->
                                    <div class="flex gap-3 flex-row items-center">
                                        <select 
                                            bind:value={selectedFormat} 
                                            class="rounded-full px-4 h-10 outline-none bg-[#404040] p-2 text-sm w-[266px]"
                                        >
                                            {#if filteredFormats && filteredFormats.length > 0}
                                                {#each filteredFormats as format}
                                                    {@const fullSizeMB = format.filesize ? (format.filesize / 1024 / 1024) : null}
                                                    {@const clipDuration = selectedClip ? timeToSeconds(selectedClip.end_time) - timeToSeconds(selectedClip.start_time) : null}
                                                    {@const videoDuration = video?.duration || null}
                                                    {@const audioFormats = video?.formats?.filter(f => 
                                                        (f.resolution === 'N/A' || f.resolution === '' || f.quality.toLowerCase().includes('audio') || ['m4a','webm','mp3','opus'].includes(f.ext)) && f.filesize
                                                    ) || []}
                                                    {@const matchingAudio = audioFormats.find(a => a.ext === format.ext) || audioFormats[0]}
                                                    {@const audioSizeMB = matchingAudio && matchingAudio.filesize ? (matchingAudio.filesize / 1024 / 1024) : 0}
                                                    {@const approxClipSizeMB = (fullSizeMB && clipDuration && videoDuration)
                                                        ? (fullSizeMB * (clipDuration / videoDuration)) + (audioSizeMB * (clipDuration / videoDuration))
                                                        : null}
                                                    <option value={format}>
                                                        {format.quality} ({format.resolution}) - {format.ext}
                                                        {#if approxClipSizeMB}
                                                            (~{approxClipSizeMB.toFixed(1)} MB)
                                                        {/if}
                                                    </option>
                                                {/each}
                                            {:else}
                                                <option value={null}>No formats available</option>
                                            {/if}
                                        </select>
                                        <button 
                                            class=" mr-2  flex items-center justify-center gap-2 rounded-full bg-white/50 hover:bg-white/90 px-2 py-2 font-medium text-black/80 disabled:opacity-50 h-10 aspect-square"
                                            disabled={downloading || !selectedFormat}
                                            onclick={() => downloadClip(selectedClip)}
                                        >
                                            {#if downloading}
                                                <Loader2 class="animate-spin" size={16} />
                                                Downloading...
                                            {:else if downloadError}
                                                <X size={16} class="text-red-500" />
                                                Failed to download
                                            {:else}
                                                <Download size={16} />
                                            {/if}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        {/if}
                    {/if}
                </div>
            </div>
        </div>
    {/if}
    
    <div class="mt-auto pt-15 flex flex-col w-full max-w-[600px] mx-auto">
        <div class="mb-3 flex gap-3 items-center justify-between">
            <button
                onclick={handlePasteClick}
                class="ml-5 flex items-center justify-center gap-3 self-end text-sm opacity-75"
            >
                {#if copied}
                    <CopyCheck class="mb-0.5" size={15} />
                {:else}
                    <Copy class="mb-0.5" size={15} />
                {/if}
                Click to Paste URL
            </button>
            {#if urlInput}
                <div class="flex items-center justify-center gap-3 text-center text-sm opacity-55">
                    {#if loading}
                        <Loader2 class="mb-0.5 animate-spin" size={15} />
                    {/if}
                    {#if video}

                        <span title={video.title}>{video.title.slice(0, 44) + ' ...'}</span>
                    {:else}
                        {urlInput.slice(0, 33) + ' ...'}
                    {/if}
                </div>
            {/if}
        </div>

        <div class="flex w-full flex-col gap-4  rounded-4xl border border-gray-500/30 p-5 dark:border-none bg-[#303030]">
            <input
                class="w-full resize-none rounded-sm border-none placeholder:text-white/70 focus:ring-0 focus:outline-none dark:placeholder:text-white/50"
                bind:value={promptInput}
                placeholder="What kind of clips do you want? (Optional)"
                onkeydown={(e) => {
                    if (e.key === 'Enter') {
                        e.preventDefault();
                        generateClips();
                    }
                }}
            />
            <div class="flex justify-between items-center">

            <div class="flex gap-3">
                <span class="text-xs text-white/50 flex items-center gap-2">Clips: ~{numClips}<input bind:value={numClips} type="range" min="1" max="9"  class="w-10 slider" /></span>
                <span class="text-xs text-white/50 flex items-center gap-2">Duration: &lt; {maxClipDuration} mins<input bind:value={maxClipDuration} type="range" min="1" max="9"  class="w-10 slider" /> </span>
                <PortionPopover 
                    videoDuration={video?.duration || 0}
                    startTime={contextStartTime}
                    endTime={contextEndTime}
                    isFullVideo={isFullVideoContext}
                    on:contextChange={handleContextChange}
                    on:transcriptFetched={handleTranscriptFetched}
                    transcript={transcript ?? ""}
                    videoId={video ? extractVideoId(video.url) : null}
                    disabled={!(urlInput && !loading)}
                />
            </div>
            <button
                disabled={!(urlInput && !loading)}
                class="rounded-[15px] bg-white/80 py-2 w-[77px] text-xs disabled:opacity-40 dark:text-black"
                onclick={() => generateClips()}
            >
                Get Clips
            </button>
            </div>
        </div>
    </div>
</div>


<!-- Settings Button -->
<div class="fixed bottom-4 left-4 flex-col flex gap-2">
    <button
        class="rounded-full bg-[#303030] p-2 hover:bg-[#404040] transition-colors"
        onclick={() => {
            showHistoryModal = true;
        }}
    >
        <Clock size={16} />
    </button>
    <button
    class="rounded-full bg-[#303030] p-2 hover:bg-[#404040] transition-colors"
    onclick={() => (showSettingsModal = true)}
        >
    <Settings  size={16} />
</button>
</div>

<!-- History Modal -->
<HistoryModal 
    show={showHistoryModal}
    videoHistory={videoHistory}
    onClose={() => (showHistoryModal = false)}
    onClearHistory={clearHistory}
    onRemoveFromHistory={removeFromHistory}
    onSelectVideo={(url) => {
        resetVideoState(); 
        urlInput = url;
        processVideo();
    }}
/>

<!-- Settings Modal -->
<SettingsModal 
    show={showSettingsModal}
    apiKey={GEMINI_API_KEY}
    downloadDir={downloadDir}
    model={model}
    showBgImage={showBgImage}
    onClose={() => (showSettingsModal = false)}
    onSave={saveSettings}
    onApiKeyChange={(value) => (GEMINI_API_KEY = value)}
    onDownloadDirChange={(value) => (downloadDir = value)}
    onModelChange={(value) => (model = value)}
    onShowBgImageChange={(value) => (showBgImage = value)}
/>

{#if downloadError}
    <div class="mt-2 text-sm text-red-500 bg-red-500/10 p-2 rounded">
        {downloadError}
    </div>
{/if}

<DownloadProgressModal downloads={downloads} onClose={removeDownload} />

<AddClipModal
    show={showAddClipModal}
    videoDuration={video?.duration || 0}
    onClose={() => (showAddClipModal = false)}
    onCreateClip={generateCustomClip}
/>

<style>
    .aspect-video {
        aspect-ratio: 16/9;
        width: 100%;
        background: #000;
        border-radius: 8px;
        overflow: hidden;
        display: flex;
        align-items: center;
        justify-content: center;
        position: relative;
    }

    ::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }

    ::-webkit-scrollbar-track {
        background: rgba(255, 255, 255, 0.1);
        border-radius: 4px;
    }

    ::-webkit-scrollbar-thumb {
        background: rgba(255, 255, 255, 0.2);
        border-radius: 4px;
    }

    ::-webkit-scrollbar-thumb:hover {
        background: rgba(255, 255, 255, 0.3);
    }

    .history-modal-content {
        max-height: 80vh;
        overflow-y: auto;
    }
</style>
