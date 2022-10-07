use error_chain::error_chain;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use url::Url;

error_chain! {
     foreign_links {
         Io(std::io::Error);
         HttpRequest(reqwest::Error);
     }
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
    let fpath = element
        .as_str()
        .split('/')
        .last()
        .expect("Fail to find file name");
    println!("The file path is : {}", fpath);
    let target = Url::parse(element).expect("Fail to convert target name to url");

    //Here is where the download happens
    let response = reqwest::get(target).await?;
    let path = Path::new(fpath);
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}", why),
        Ok(file) => file,
    };
    let content = response.bytes().await?;
    file.write_all(&content)?;

    println!("Download succeeded");

    Ok(())
}
