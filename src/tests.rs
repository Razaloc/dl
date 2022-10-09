#[cfg(test)]
use super::*;

#[tokio::test]
async fn test_http() -> Result<()> {
    let element = "http://adaptive-images.com/content/images/winter.jpg".to_string();
    let (url, fpath) = setup(&element);
    fetch_url_http(url, fpath).await?;
    let mut file1 = std::fs::File::open(fpath)?;
    let mut file2 = std::fs::File::open("./tests/data/".to_string() + fpath)?;
    match file_diff::diff_files(&mut file1, &mut file2) {
        true => std::fs::remove_file(fpath).expect("File delete failed"),
        false => panic!("The downloaded file aint the same as the tests/data/ one"),
    };
    Ok(())
}

#[tokio::test]
async fn test_https() -> Result<()> {
    let element = "https://www.rust-lang.org/logos/rust-logo-128x128-blk.png".to_string();
    let (url, fpath) = setup(&element);
    fetch_url_https(url, fpath).await?;
    let mut file1 = std::fs::File::open(fpath)?;
    let mut file2 = std::fs::File::open("./tests/data/".to_string() + fpath)?;
    match file_diff::diff_files(&mut file1, &mut file2) {
        true => std::fs::remove_file(fpath).expect("File delete failed"),
        false => panic!("The downloaded file aint the same as the tests/data/ one"),
    };
    Ok(())
}
