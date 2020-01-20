use anyhow::Result;
use instascrape::scrape;

#[tokio::test]
async fn test_scrape() -> Result<()> {
    scrape("arcturusrobotics").await?;

    Ok(())
}
