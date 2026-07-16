use serde::Deserialize;

/// Attachment metadata as returned in message, thread, and draft responses.
/// `download_url` (a short-lived presigned URL) is populated only by the
/// attachment-download endpoints; the other fields describe the attachment in
/// list and get responses.
#[derive(Clone, Debug, Deserialize)]
pub struct Attachment {
    /// Unique attachment id.
    pub attachment_id: String,
    /// File name.
    #[serde(default)]
    pub filename: Option<String>,
    /// Size in bytes.
    #[serde(default)]
    pub size: Option<u64>,
    /// MIME content type.
    #[serde(default)]
    pub content_type: Option<String>,
    /// Content-Disposition (e.g. `inline` or `attachment`).
    #[serde(default)]
    pub content_disposition: Option<String>,
    /// Content-ID, for inline attachments referenced by the HTML body.
    #[serde(default)]
    pub content_id: Option<String>,
    /// Short-lived presigned URL to download the bytes; populated only by the
    /// attachment-download endpoints.
    #[serde(default)]
    pub download_url: Option<String>,
    /// When `download_url` expires (RFC 3339).
    #[serde(default)]
    pub expires_at: Option<String>,
}
