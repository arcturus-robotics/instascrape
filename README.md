# Instascrape

An Instagram scraper created to help keep track of our Instagram followers.

## Usage

```text
USAGE:
    instascrape-cli [OPTIONS] --interval <interval> --output <output> --user <user>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --interval <interval>    The interval in seconds at which to scrape
    -o, --output <output>        The path of the file to output data to
    -u, --user <user>            The Instagram user to scrape data from
    -w, --webhook <webhook>      A Discord webhook to send messages to [env: INSTASCRAPE_WEBHOOK]
```
