use anyhow::{Context, Result};
use argh::FromArgs;
use regex::Regex;
use std::io::{Read, Write};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use tokio::sync::Semaphore;
use std::sync::Arc;
use futures_util::StreamExt;

#[derive(FromArgs)]
#[argh(description = "Download a file from a URL")]
struct Command {
    /// the URL from fuckingfast.co to download
    #[argh(option, short = 'u')]
    url: Option<String>,

    /// load urls to download from a text file
    #[argh(option, long = "file", short = 'f')]
    file: Option<String>,

    /// number of concurrent downloads
    #[argh(option, short = 'c', default = "4")]
    concurrent: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let command: Command = argh::from_env();
    let semaphore = Arc::new(Semaphore::new(command.concurrent));
    let multi_progress = MultiProgress::new();

    if let Some(url) = command.url {
        let download_url = fetch_download_url(&url).await?;
        download_file(&download_url, &multi_progress).await?;
    }

    if let Some(filename) = command.file {
        println!("Fetching urls from: {}", filename);
        let urls = get_file_urls(&filename).await?;
        let mut handles = vec![];

        for url in urls {
            let download_url = fetch_download_url(&url).await?;
            let sem = semaphore.clone();
            let multi_progress = multi_progress.clone();
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                download_file(&download_url, &multi_progress).await
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await??;
        }
    }

    Ok(())
}

async fn get_file_urls(filename: &str) -> Result<Vec<String>> {
    let file = std::fs::File::open(filename).context("Failed to open file")?;
    let mut reader = std::io::BufReader::new(file);
    let mut content = String::new();
    reader.read_to_string(&mut content).context("Failed to read file")?;
    
    let re = Regex::new(r#"https?://[^\s"<>]+"#).context("Failed to create regex")?;
    let urls: Vec<String> = re.find_iter(&content)
        .map(|m| m.as_str().trim_matches('"').to_string())
        .collect();
    
    Ok(urls)
}

async fn fetch_download_url(url: &str) -> Result<String> {
    let response = reqwest::get(url).await.context("Failed to get response from url")?;
    let html_content = response.text().await.context("Failed to get html content")?;

    let re = Regex::new(r#"window\.open\("([^"]+fuckingfast\.co/dl/[^"]+)""#).context("Failed to create regex")?;
    
    if let Some(capture) = re.captures(&html_content) {
        Ok(capture[1].to_string())
    } else {
        Err(anyhow::anyhow!("No download link found"))
    }
}

async fn download_file(url: &str, multi_progress: &MultiProgress) -> Result<()> {
    let response = reqwest::get(url).await.context("Failed to get response")?;
    let total_size = response.content_length().unwrap_or(0);
    let headers = response.headers().clone();
    
    let filename = headers
        .get("content-disposition")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split("filename*=UTF-8''").nth(1))
        .map(|s| s.trim_matches('"'))
        .context("No filename in headers")?;
    
    let pb = multi_progress.add(ProgressBar::new(total_size));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}) - {msg}")
        .unwrap());
    pb.set_message(filename.to_string());
    
    let mut file = std::fs::File::create(filename).context("Failed to create file")?;
    let mut downloaded = 0;
    
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to get chunk")?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    
    pb.finish_with_message(format!("{} - Complete", filename));
    Ok(())
}