use anyhow::Result;
use instascrape::scrape;

#[tokio::test]
async fn test_scrape() -> Result<()> {
    let data = scrape("arcturusrobotics").await?;

    Ok(())
}
