use anyhow::Context;
use reqwest::{Client, Url};
use std::{sync::OnceLock, time::Duration};

pub(crate) fn client() -> anyhow::Result<Client> {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    if let Some(client) = CLIENT.get() {
        return Ok(client.clone());
    }
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(15))
        .build()
        .context("Failed to initialize HTTP client")?;
    Ok(CLIENT.get_or_init(|| client).clone())
}

pub(crate) async fn get(client: &Client, url: Url) -> anyhow::Result<Vec<u8>> {
    let mut response = client
        .get(url)
        .send()
        .await
        .context("Failed to contact remote service")?
        .error_for_status()
        .context("Remote service returned an unsuccessful HTTP status")?;
    const MAX_BYTES: usize = 8 * 1024 * 1024;
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .context("Failed to read remote response")?
    {
        anyhow::ensure!(
            body.len() + chunk.len() <= MAX_BYTES,
            "Remote response exceeds size limit"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    async fn server(status: u16, delay: Duration) -> (Url, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            stream.read(&mut request).await.unwrap();
            tokio::time::sleep(delay).await;
            let response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Length: 4\r\nConnection: close\r\n\r\nbody"
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
        (url, task)
    }

    #[tokio::test]
    async fn error_statuses_are_rejected_before_parsing() {
        let client = Client::builder().no_proxy().build().unwrap();
        for status in [403, 429, 500] {
            let (url, server) = server(status, Duration::ZERO).await;
            let error = get(&client, url).await.unwrap_err();
            assert_eq!(
                error
                    .downcast_ref::<reqwest::Error>()
                    .unwrap()
                    .status()
                    .unwrap()
                    .as_u16(),
                status
            );
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn slow_responses_time_out() {
        let client = Client::builder()
            .no_proxy()
            .timeout(Duration::from_millis(50))
            .build()
            .unwrap();
        let (url, server) = server(200, Duration::from_secs(2)).await;
        let error = get(&client, url).await.unwrap_err();
        assert!(error.downcast_ref::<reqwest::Error>().unwrap().is_timeout());
        server.abort();
    }

    #[tokio::test]
    async fn successful_responses_return_the_body() {
        let (url, server) = server(200, Duration::ZERO).await;
        let client = Client::builder().no_proxy().build().unwrap();
        assert_eq!(get(&client, url).await.unwrap(), b"body");
        server.await.unwrap();
    }
}
