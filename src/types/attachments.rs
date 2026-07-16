use serde::{Deserialize, Serialize};

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

/// An attachment to include on an outgoing message or draft. Supply the bytes
/// inline as base64 `content`, or a `url` for the API to fetch; set `content_id`
/// to reference the attachment inline from the HTML body.
#[derive(Clone, Debug, Default, Serialize)]
pub struct SendAttachment {
    /// File name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    /// MIME content type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Content-Disposition (e.g. `inline` or `attachment`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,
    /// Content-ID, to reference this attachment inline from the HTML body.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    /// The attachment bytes, base64-encoded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// A URL for the API to fetch the attachment from, instead of `content`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}
