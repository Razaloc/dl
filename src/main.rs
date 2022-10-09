mod tests;
use error_chain::error_chain;
use hyper::body::HttpBody;
use hyper::header::{CONTENT_LENGTH, RANGE};
use hyper::http::HeaderValue;
use hyper::{Client, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use std::fs::File;
use std::str::FromStr;
use std::{env, io::Write};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        Header(reqwest::header::ToStrError);
    }
}
struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self> {
        if buffer_size == 0 {
            Err("invalid buffer_size, give a value greater than zero.")?;
        }
        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some(
                HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                    .expect("string provided by format!"),
            )
        }
    }
}
type Result1<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn setup(element: &str) -> (Uri, &str) {
    let fpath = element.split('/').last().expect("Fail to find file name");
    println!("\nThe file path is : {}", fpath);
    let url = element
        .parse::<Uri>()
        .expect("Fail to convert target name to url");
    (url, fpath)
}

#[tokio::main]
async fn main() -> Result1<()> {
    //Setting up argument as filename and url
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("No arguments given");
        return Ok(());
    }

    let element = &args[1];
    let (url, fpath) = setup(element);

    //Call to dowload functions
    match url.scheme_str() {
        Some("https") => fetch_url_https(url.clone(), fpath).await?,
        Some("http") => fetch_url_http(url, fpath).await?,
        _ => {
            println!("url is not http or https");
        }
    }
    Ok(())
}

pub async fn fetch_url_http(url: hyper::Uri, fpath: &str) -> Result1<()> {
    let client = Client::new();
    let mut res = client.get(url).await?;
    let mut file = std::fs::File::create(fpath).unwrap();

    while let Some(next) = res.data().await {
        let chunk = next?;
        file.write_all(&chunk)?;
    }

    println!("\nDone!");

    Ok(())
}

pub async fn fetch_url_https(url: hyper::Uri, fpath: &str) -> Result1<()> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut res = client.get(url).await?;
    let mut file = std::fs::File::create(fpath).unwrap();

    while let Some(next) = res.data().await {
        let chunk = next?;
        file.write_all(&chunk)?;
    }

    println!("\n\nDone!");

    Ok(())
}

pub async fn fetch_url_http_range_requests(url: hyper::Uri, fpath: &str) -> Result<()> {
    const CHUNK_SIZE: u32 = 10240;
    let client = reqwest::Client::new();
    let url = url.to_string();
    let response = client.head(&url).send().await?;
    let length = response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or("response doesn't include the content length")?;
    let length = u64::from_str(length.to_str()?).map_err(|_| "invalid Content-Length header")?;

    let mut output_file = File::create(fpath)?;

    println!("starting download...");
    for range in PartialRangeIter::new(0, length - 1, CHUNK_SIZE)? {
        println!("range {:?}", range);
        let inner_response = client.get(&url).header(RANGE, range).send().await?;

        println!("------------> {:?} ", inner_response.content_length());
        let status = inner_response.status();
        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
            error_chain::bail!("Unexpected server response: {}", status)
        }
        let content = &inner_response.bytes().await?;
        output_file.write_all(content)?;
    }
    let content = response.bytes().await?;
    output_file.write_all(&content)?;
    println!("Finished with success!");
    Ok(())
}
