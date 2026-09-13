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
pub(crate) mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    const TEST_TIMEOUT: Duration = Duration::from_secs(5);

    pub(crate) fn client_builder() -> reqwest::ClientBuilder {
        // These tests use plain HTTP on loopback. Do not depend on host proxies
        // or system certificate roots, which may be absent in a Nix sandbox.
        Client::builder()
            .no_proxy()
            .tls_certs_only([])
            .connect_timeout(TEST_TIMEOUT)
            .timeout(TEST_TIMEOUT)
    }

    pub(crate) fn client() -> Client {
        client_builder().build().unwrap()
    }

    pub(crate) struct TestServer {
        task: tokio::task::JoinHandle<std::io::Result<()>>,
    }

    impl TestServer {
        pub(crate) async fn finish(self) -> anyhow::Result<()> {
            self.finish_with_timeout(TEST_TIMEOUT).await
        }

        async fn finish_with_timeout(mut self, timeout: Duration) -> anyhow::Result<()> {
            tokio::time::timeout(timeout, &mut self.task)
                .await
                .context("Timed out waiting for local HTTP test server")?
                .context("Local HTTP test server task failed")?
                .context("Local HTTP test server I/O failed")
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn server(status: u16, delay: Duration) -> (Url, TestServer) {
        server_with_body(status, delay, "body").await
    }

    pub(crate) async fn server_with_body(
        status: u16, delay: Duration, body: impl AsRef<[u8]>,
    ) -> (Url, TestServer) {
        let body = body.as_ref().to_vec();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = Url::parse(&format!("http://{}/", listener.local_addr().unwrap())).unwrap();
        let task = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await?;
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut chunk = [0; 1024];
                let count = stream.read(&mut chunk).await?;
                if count == 0 {
                    return Err(std::io::ErrorKind::UnexpectedEof.into());
                }
                request.extend_from_slice(&chunk[..count]);
                assert!(request.len() <= 16384, "Request headers exceed test limit");
            }
            tokio::time::sleep(delay).await;
            let mut response = format!(
                "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes();
            response.extend_from_slice(&body);
            stream.write_all(&response).await
        });
        (url, TestServer { task })
    }

    #[tokio::test]
    async fn error_statuses_are_rejected_before_parsing() {
        let client = client();
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
            server.finish().await.unwrap();
        }
    }

    #[tokio::test]
    async fn slow_responses_time_out() {
        let client = client_builder()
            .timeout(Duration::from_millis(50))
            .build()
            .unwrap();
        let (url, server) = server(200, Duration::from_secs(2)).await;
        let error = get(&client, url).await.unwrap_err();
        assert!(error.downcast_ref::<reqwest::Error>().unwrap().is_timeout());
        drop(server);
    }

    #[tokio::test]
    async fn successful_responses_return_the_body() {
        let (url, server) = server(200, Duration::ZERO).await;
        let client = client();
        assert_eq!(get(&client, url).await.unwrap(), b"body");
        server.finish().await.unwrap();
    }

    #[tokio::test]
    async fn server_wait_is_bounded_when_no_request_arrives() {
        let (_, server) = server(200, Duration::ZERO).await;
        let task = server.task.abort_handle();
        let error = server
            .finish_with_timeout(Duration::from_millis(20))
            .await
            .unwrap_err();
        assert!(error.is::<tokio::time::error::Elapsed>());
        tokio::task::yield_now().await;
        assert!(task.is_finished());
    }

    #[tokio::test]
    async fn dropping_server_cancels_an_incomplete_request() {
        let (url, server) = server(200, Duration::ZERO).await;
        let mut stream = tokio::net::TcpStream::connect(url.socket_addrs(|| None).unwrap()[0])
            .await
            .unwrap();
        stream.write_all(b"GET / HTTP/1.1\r\n").await.unwrap();
        let task = server.task.abort_handle();
        drop(server);
        tokio::task::yield_now().await;
        assert!(task.is_finished());
    }

    #[tokio::test]
    async fn speech_test_ignores_proxy_and_certificate_environment() {
        // Run in a child process so environment changes cannot race other tests.
        let root = tempfile::tempdir().unwrap();
        let certificates = root.path().join("empty-certificates.pem");
        std::fs::write(&certificates, b"").unwrap();
        let mut command = tokio::process::Command::new(std::env::current_exe().unwrap());
        command.args([
            "--exact",
            "word::speech::tests::invalid_audio_responses_are_never_cached",
            "--nocapture",
        ]);
        for key in [
            "http_proxy",
            "HTTP_PROXY",
            "https_proxy",
            "HTTPS_PROXY",
            "all_proxy",
            "ALL_PROXY",
        ] {
            command.env(key, "http://127.0.0.1:1");
        }
        for key in ["no_proxy", "NO_PROXY"] {
            command.env(key, "");
        }
        command
            .env("SSL_CERT_FILE", certificates)
            .env("SSL_CERT_DIR", root.path())
            .kill_on_drop(true);
        let output = tokio::time::timeout(Duration::from_secs(10), command.output())
            .await
            .expect("Speech regression test exceeded its deadline")
            .unwrap();
        assert!(
            output.status.success(),
            "Speech regression failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}
