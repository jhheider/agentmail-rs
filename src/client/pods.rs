use crate::client::NoBody;
use crate::{Client, Error, Page, types::*, util::urlish};

impl Client {
    /// POST /v0/pods, create a pod.
    pub async fn create_pod(&self, pod: CreatePod) -> Result<Pod, Error> {
        self.request(reqwest::Method::POST, "/v0/pods", &[], Some(&pod))
            .await
    }

    /// GET /v0/pods, one page.
    pub async fn list_pods(&self, page: Page) -> Result<PodList, Error> {
        self.request(
            reqwest::Method::GET,
            "/v0/pods",
            &page.query(),
            None::<&NoBody>,
        )
        .await
    }

    /// Every pod, draining pagination.
    pub async fn list_all_pods(&self) -> Result<Vec<Pod>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_pods(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.pods);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
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
