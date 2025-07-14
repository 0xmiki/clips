
use dirs;
use roxmltree;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
// use tauri::Manager; // Removed unused import

#[cfg(windows)]
use std::os::windows::process::CommandExt;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Format {
    pub format_id: String,
    pub ext: String,
    pub quality: String,
    pub filesize: Option<i64>,
    pub resolution: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    pub url: String,
    pub title: String,
    pub author: String,
    pub duration: i64,
    pub thumbnail: String,
    pub transcript: Option<String>,
    pub formats: Vec<Format>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamingUrl {
    pub video_url: String,
    pub streaming_url: String,
    pub expire_date: String,
}

#[tauri::command]
async fn extract_video_metadata(_app: tauri::AppHandle, url: String) -> Result<Video, String> {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-j")
        .arg("--no-check-certificates")
        .arg("--no-warnings")
        .arg(&url);
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000); 
    }
    
    // Use stderr to capture progress and errors, but don't fail on warnings
    let output = cmd
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;

    let output = output
        .wait_with_output()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;

    // Check if we got valid JSON output, regardless of exit status
    // yt-dlp might exit with non-zero status but still provide valid metadata
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    if stdout_str.trim().is_empty() {
        let error_msg = format!(
            "yt-dlp failed to extract metadata: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(error_msg);
    }

    let meta: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| {
            let error_msg = format!("Failed to parse yt-dlp output: {e}");
            error_msg
        })?;

    let title = meta["title"].as_str().unwrap_or("").to_string();
    let author = meta["uploader"].as_str().unwrap_or("").to_string();
    let duration = meta["duration"].as_i64().unwrap_or(0);
    let thumbnail = meta["thumbnail"].as_str().unwrap_or("").to_string();

    let formats = meta["formats"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|f| {
            let format_id = f["format_id"].as_str()?.to_string();
            let ext = f["ext"].as_str()?.to_string();
            let quality = f["format_note"].as_str().unwrap_or("unknown").to_string();
            let filesize = f["filesize"].as_i64();
            let width = f["width"].as_i64().unwrap_or(0);
            let height = f["height"].as_i64().unwrap_or(0);
            let resolution = if width > 0 && height > 0 {
                format!("{}x{}", width, height)
            } else {
                "N/A".to_string()
            };

            Some(Format {
                format_id,
                ext,
                quality,
                filesize,
                resolution,
            })
        })
        .collect();


    Ok(Video {
        url: url.clone(),
        title,
        author,
        duration,
        thumbnail,
        transcript: None,
        formats,
    })
}

#[tauri::command]
async fn fetch_transcript_backend(_app: tauri::AppHandle, video_id: String) -> Result<String, String> {
    let lang = "en";
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--write-auto-sub")
        .arg("--skip-download")
        .arg("--sub-lang")
        .arg(lang)
        .arg("--sub-format")
        .arg("srv1")
        .arg("--no-check-certificates")
        .arg("--no-warnings")
        .arg("-o")
        .arg("%(id)s.%(ext)s")
        .arg(&format!("https://www.youtube.com/watch?v={}", video_id));
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000);
    }
    
    // Use stderr to capture progress and errors, but don't fail on warnings
    let output = cmd
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;

    let output = output
        .wait_with_output()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;

    // Check if transcript file was created, regardless of exit status
    let fname = format!("{}.en.srv1", video_id);
    if let Ok(txt) = std::fs::read_to_string(&fname) {
        let _ = std::fs::remove_file(&fname);

        let mut lines = Vec::new();
        if let Ok(doc) = roxmltree::Document::parse(&txt) {
            for node in doc.descendants().filter(|n| n.has_tag_name("text")) {
                let start = node.attribute("start").unwrap_or("");
                let text = node
                    .text()
                    .unwrap_or("")
                    .replace("\n", " ")
                    .replace("  ", " ");

                let secs = start.parse::<f32>().unwrap_or(0.0);
                let mins = (secs / 60.0).floor() as u32;
                let secs = (secs % 60.0).round() as u32;
                let timestamp = format!("{}:{:02}", mins, secs);
                lines.push(format!("{}\n{}", timestamp, text));
            }
        }
        Ok(lines.join("\n"))
    } else {
        // If no transcript file was created, then it's a real error
        let error_msg = format!(
            "Could not read transcript file. yt-dlp output: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(error_msg);
    }
}

#[tauri::command]
async fn get_streaming_url(_app: tauri::AppHandle, video_url: String) -> Result<StreamingUrl, String> {

    let mut cmd = Command::new("yt-dlp");
    cmd.arg("-f")
        .arg("best[height<=720]") 
        .arg("-g")
        .arg("--no-check-certificates")
        .arg("--no-warnings")
        .arg(&video_url);
    #[cfg(windows)]
    {
        cmd.creation_flags(0x08000000); 
    }
    
    // Use stderr to capture progress and errors, but don't fail on warnings
    let output = cmd
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| {
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;

    let output = output
        .wait_with_output()
        .await
        .map_err(|e| {
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;

    // Check if we got a streaming URL, regardless of exit status
    let streaming_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if streaming_url.is_empty() {
        let error_msg = format!(
            "No streaming URL found. yt-dlp output: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(error_msg);
    }

    let expire_date = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(6))
        .unwrap_or_else(|| chrono::Utc::now())
        .to_rfc3339();


    Ok(StreamingUrl {
        video_url,
        streaming_url,
        expire_date,
    })
}


enum DownloadType {
    Clip { start: String, end: String },
    Full,
}


async fn download_video_internal(
    app: tauri::AppHandle,
    video_url: String,
    format_id: String,
    output_dir: String,
    filename: String,
    download_type: DownloadType,
) -> Result<String, String> {

    let expanded_dir = if output_dir.starts_with("~") {
        match dirs::home_dir() {
            Some(mut path) => {
                path.push(&output_dir[2..]);
                path.to_string_lossy().into_owned()
            }
            None => output_dir,
        }
    } else {
        output_dir
    };

    let temp_dir = format!("{}/temp_{}", expanded_dir, chrono::Utc::now().timestamp());
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| {
            let _function_name = match &download_type {
                DownloadType::Clip { .. } => "download_clip",
                DownloadType::Full => "download_full_video",
            };
            let error_msg = format!("Failed to create temporary directory: {e}");
            error_msg
        })?;


    let temp_path = format!("{}/temp.%(ext)s", temp_dir);

    let clean_filename = filename
        .chars()
        .map(|c| match c {
            ' ' | '_' | '-' => '_',
            c if c.is_alphanumeric() => c.to_ascii_lowercase(),
            _ => '_',
        })
        .collect::<String>();
    let clean_filename = clean_filename
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    let clean_filename = match &download_type {
        DownloadType::Clip { .. } => {
            if clean_filename.len() > 100 {
                format!("{}_clip", &clean_filename[..100])
            } else {
                format!("{}_clip", clean_filename)
            }
        }
        DownloadType::Full => {
            if clean_filename.len() > 100 {
                format!("{}_full", &clean_filename[..100])
            } else {
                format!("{}_full", clean_filename)
            }
        }
    };

    let final_path = format!("{}/{}.mp4", expanded_dir, clean_filename);

    app.emit(
        "download_progress",
        serde_json::json!({"status": "processing"}),
    )
    .ok();
    let mut yt_dlp_cmd = Command::new("yt-dlp");
    yt_dlp_cmd.arg("-f")
        .arg(format!("{format_id}+bestaudio"));
    match &download_type {
        DownloadType::Clip { start, end } => {
            yt_dlp_cmd
                .arg("--download-sections")
                .arg(&format!("*{}-{}", start, end));
        }
        DownloadType::Full => {}
    }
    yt_dlp_cmd
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("--no-check-certificates")
        .arg("--no-warnings")
        .arg("--add-header")
        .arg("referer:youtube.com")
        .arg("--add-header")
        .arg("user-agent:Mozilla/5.0")
        .arg("-o")
        .arg(&temp_path)
        .arg(&video_url)
        .arg("--newline");
    #[cfg(windows)]
    {
        yt_dlp_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let mut yt_dlp_cmd = yt_dlp_cmd
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .map_err(|e| {
            let _function_name = match &download_type {
                DownloadType::Clip { .. } => "download_clip",
                DownloadType::Full => "download_full_video",
            };
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;
    if let Some(stderr) = yt_dlp_cmd.stderr.take() {
        let mut reader = BufReader::new(stderr);
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await.unwrap_or(0);
            if bytes_read == 0 {
                break;
            }
            let line_trimmed = line.trim();
            if line_trimmed.contains("[download]") && line_trimmed.contains("%") {
                let percent = line_trimmed
                    .split_whitespace()
                    .find(|s| s.ends_with('%'))
                    .and_then(|s| s.trim_end_matches('%').parse::<f32>().ok());
                let speed = line_trimmed
                    .split_whitespace()
                    .find(|s| s.ends_with("/s"))
                    .map(|s| s.to_string());
                let eta = line_trimmed
                    .split_whitespace()
                    .rev()
                    .find(|s| s.chars().all(|c| c.is_digit(10) || c == ':'))
                    .map(|s| s.to_string());
                app.emit(
                    "download_progress",
                    serde_json::json!({
                        "status": "downloading",
                        "percent": percent,
                        "speed": speed,
                        "eta": eta
                    }),
                )
                .ok();
            }
        }
    }
    let output = yt_dlp_cmd
        .wait_with_output()
        .await
        .map_err(|e| {
            let _function_name = match &download_type {
                DownloadType::Clip { .. } => "download_clip",
                DownloadType::Full => "download_full_video",
            };
            let error_msg = format!("Failed to run yt-dlp: {e}");
            error_msg
        })?;
    
    // Check if a file was actually downloaded, regardless of exit status
    let temp_file = std::fs::read_dir(&temp_dir)
        .map_err(|e| {
            let _function_name = match &download_type {
                DownloadType::Clip { .. } => "download_clip",
                DownloadType::Full => "download_full_video",
            };
            let error_msg = format!("Failed to read temp directory: {e}");
            error_msg
        })?
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_string_lossy().starts_with("temp."));
    
    if temp_file.is_none() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _function_name = match &download_type {
            DownloadType::Clip { .. } => "download_clip",
            DownloadType::Full => "download_full_video",
        };
        let error_msg = format!(
            "yt-dlp failed to download file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        app.emit("download_progress", serde_json::json!({"status": "error", "error": String::from_utf8_lossy(&output.stderr)})).ok();
        return Err(error_msg);
    }
    
    let temp_file = temp_file.unwrap();
    app.emit(
        "download_progress",
        serde_json::json!({"status": "processing"}),
    )
    .ok();
    let mut ffmpeg_cmd = Command::new("ffmpeg");
    // Doesn't work on ios
    // ffmpeg_cmd.arg("-i")
    //     .arg(temp_file.path().to_str().unwrap())
    //     .arg("-c")
    //     .arg("copy")
    //     .arg("-movflags")
    //     .arg("+faststart")
    //     .arg("-y")
    //     .arg(&final_path);
    ffmpeg_cmd
        .arg("-i")
        .arg(temp_file.path().to_str().unwrap())
        // --- Video ---
        .arg("-c:v")
        .arg("libx264") // force H.264
        .arg("-profile:v")
        .arg("baseline") // safest profile for every iPhone/iPad
        .arg("-level")
        .arg("4.0") // high enough for 1080 p25, low enough for older devices
        .arg("-preset")
        .arg("fast") // speed vs. size trade-off (use "medium" if you want smaller files)
        .arg("-crf")
        .arg("23") // Constant-quality 0-51 (lower = bigger). 18-23 is visually fine.
        // --- Audio ---
        .arg("-c:a")
        .arg("aac") // iOS always supports AAC
        .arg("-b:a")
        .arg("128k") // stereo 128 kb/s
        .arg("-ac")
        .arg("2") // force 2 channels
        // --- Container tweaks ---
        .arg("-movflags")
        .arg("+faststart") // moov atom at the front → quick start in Safari
        .arg("-y") // overwrite existing file
        .arg(&final_path);
        
    #[cfg(windows)]
    {
        ffmpeg_cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let ffmpeg_output = ffmpeg_cmd
        .output()
        .await
        .map_err(|e| {
            let _function_name = match &download_type {
                DownloadType::Clip { .. } => "download_clip",
                DownloadType::Full => "download_full_video",
            };
            let error_msg = format!("Failed to run ffmpeg: {e}");
            error_msg
        })?;
    let _ = std::fs::remove_dir_all(&temp_dir);
    if !ffmpeg_output.status.success() {
        let _function_name = match &download_type {
            DownloadType::Clip { .. } => "download_clip",
            DownloadType::Full => "download_full_video",
        };
        let error_msg = format!(
            "ffmpeg failed: {}",
            String::from_utf8_lossy(&ffmpeg_output.stderr)
        );
        app.emit("download_progress", serde_json::json!({"status": "error", "error": error_msg})).ok();
        return Err(error_msg);
    }
    if !std::path::Path::new(&final_path).exists() {
        let _function_name = match &download_type {
            DownloadType::Clip { .. } => "download_clip",
            DownloadType::Full => "download_full_video",
        };
        let error_msg = "Final converted file not found".to_string();
        app.emit(
            "download_progress",
            serde_json::json!({"status": "error", "error": "Final converted file not found"}),
        )
        .ok();
        return Err(error_msg);
    }
    app.emit("download_progress", serde_json::json!({"status": "done"}))
        .ok();
    let _function_name = match &download_type {
        DownloadType::Clip { .. } => "download_clip",
        DownloadType::Full => "download_full_video",
    };
  
    Ok("Download and conversion completed successfully".to_string())
}

#[tauri::command]
async fn download_clip(
    app: tauri::AppHandle,
    video_url: String,
    format_id: String,
    start_time: String,
    end_time: String,
    output_dir: String,
    filename: String,
) -> Result<String, String> {
    download_video_internal(
        app,
        video_url,
        format_id,
        output_dir,
        filename,
        DownloadType::Clip { start: start_time, end: end_time },
    ).await
}

#[tauri::command]
async fn download_full_video(
    app: tauri::AppHandle,
    video_url: String,
    format_id: String,
    output_dir: String,
    filename: String,
) -> Result<String, String> {
    download_video_internal(
        app,
        video_url,
        format_id,
        output_dir,
        filename,
        DownloadType::Full,
    ).await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            extract_video_metadata,
            fetch_transcript_backend,
            get_streaming_url,
            download_clip,
            download_full_video,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
          
            let error_msg = format!("Failed to run Tauri application: {}", e);
            eprintln!("{}", error_msg);
            std::process::exit(1);
        });
}
