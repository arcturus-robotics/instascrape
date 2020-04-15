use instascrape::scrape;

#[tokio::test]
async fn test_scrape() -> instascrape::Result<()> {
    scrape("arcturusrobotics").await?;

    Ok(())
}
