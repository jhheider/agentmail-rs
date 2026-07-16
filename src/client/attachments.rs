use crate::{Client, Error, types::*};

impl Client {
    /// Download an attachment's bytes from its presigned `download_url`. The URL
    /// is a short-lived S3 link fetched without the API bearer token, so obtain
    /// the [`Attachment`] from one of the `get_*_attachment` calls first (a
    /// plain response attachment carries no `download_url` and yields
    /// [`Error::NoDownloadUrl`]).
    pub async fn download_attachment(&self, attachment: &Attachment) -> Result<Vec<u8>, Error> {
        let url = attachment
            .download_url
            .as_deref()
            .ok_or(Error::NoDownloadUrl)?;
        self.download_url(url).await
    }

    /// Download the raw `.eml` bytes from a [`RawMessage`]'s presigned URL.
    pub async fn download_raw(&self, raw: &RawMessage) -> Result<Vec<u8>, Error> {
        self.download_url(&raw.download_url).await
    }

    /// GET a presigned URL without the API bearer token (it is already an
    /// authenticated URL) and return the bytes.
    async fn download_url(&self, url: &str) -> Result<Vec<u8>, Error> {
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;
        if !status.is_success() {
            return Err(Error::Api {
                status,
                body: String::from_utf8_lossy(&bytes).into_owned(),
            });
        }
        Ok(bytes.to_vec())
    }
}
