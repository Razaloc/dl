mod tests;
use hyper::body::HttpBody;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use std::{env, io::Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub fn setup(element: &str) -> (Uri, &str) {
    let fpath = element.split('/').last().expect("Fail to find file name");
    println!("\nThe file path is : {}", fpath);
    let url = element
        .parse::<Uri>()
        .expect("Fail to convert target name to url");
    (url, fpath)
}

#[tokio::main]
async fn main() -> Result<()> {
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

pub async fn fetch_url_http(url: hyper::Uri, fpath: &str) -> Result<()> {
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

pub async fn fetch_url_https(url: hyper::Uri, fpath: &str) -> Result<()> {
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
