use crate::{types::*, util::urlish, Error, Page};

impl super::Client {
    /// POST /v0/pods
    ///
    /// Create a new pod -- an organizational container for inboxes, domains,
    /// and webhooks with its own billing.
    pub async fn create_pod(&self, pod: CreatePod) -> Result<Pod, Error> {
        self.request(
            reqwest::Method::POST,
            "/v0/pods",
            &[],
            Some(serde_json::to_value(pod).expect("serializable")),
        )
        .await
    }

    /// GET /v0/pods
    ///
    /// List all pods in the organization.
    pub async fn list_pods(&self) -> Result<PodList, Error> {
        self.request(reqwest::Method::GET, "/v0/pods", &[], None)
            .await
    }

    /// GET /v0/pods (paginated).
    pub async fn list_pods_page(&self, page: Page) -> Result<PodList, Error> {
        self.request(reqwest::Method::GET, "/v0/pods", &page.query(), None)
            .await
    }

    /// GET /v0/pods/{pod_id}
    pub async fn get_pod(&self, pod_id: &str) -> Result<Pod, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/pods/{}", urlish(pod_id)),
            &[],
            None,
        )
        .await
    }

    /// DELETE /v0/pods/{pod_id}
    pub async fn delete_pod(&self, pod_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/pods/{}", urlish(pod_id)),
            &[],
            None,
        )
        .await
    }
}
