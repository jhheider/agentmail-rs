use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/pods, create a pod.
    pub async fn create_pod(&self, pod: CreatePod) -> Result<Pod, Error> {
        self.request(reqwest::Method::POST, "/v0/pods", &[], Some(&pod))
            .await
    }

    /// GET /v0/pods (first page; see [`Client::list_pods_page`]).
    pub async fn list_pods(&self) -> Result<PodList, Error> {
        self.list_pods_page(Page::default()).await
    }

    /// GET /v0/pods with pagination. Feed [`PodList::next_page_token`] back in
    /// as [`Page::page_token`] until it comes back `None`.
    pub async fn list_pods_page(&self, page: Page) -> Result<PodList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/pods",
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// GET /v0/pods/{pod_id}
    pub async fn get_pod(&self, pod_id: &str) -> Result<Pod, Error> {
        self.request(
            reqwest::Method::GET,
            &format!("/v0/pods/{}", urlish(pod_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }

    /// DELETE /v0/pods/{pod_id}
    pub async fn delete_pod(&self, pod_id: &str) -> Result<(), Error> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/v0/pods/{}", urlish(pod_id)),
            &[],
            None::<&NoBody>,
        )
        .await
    }
}
