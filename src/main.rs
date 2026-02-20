use clap::Parser;
use std::fs::File;
use std::io::Write;
use reqwest::Client;
use indicatif::{ProgressStyle, ProgressBar};
use futures_util::StreamExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File URL
    url: String,

    /// Output file
    #[arg(short, long)]
    output: Option<String>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file_name = args.output.unwrap_or_else(|| {
        args.url
            .split('/')
            .last()
            .filter(|s| !s.is_empty())
            .unwrap_or("download.dlm")
            .to_string()  
    });

    let client = Client::new();
    let res = client.get(&args.url).send().await?;

    if !res.status().is_success() {
        return Err(format!("Server Error: {}", res.status()).into());
    }

    let total_size = res.content_length().ok_or("The download size could not be obtained.")?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
        .progress_chars("#>-"));
    pb.set_message(format!("Saving in: {}", file_name));

    let mut file = File::create(&file_name)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;

        let new = std::cmp::min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!("Done, file saved as {}", file_name));
    Ok(())
}
