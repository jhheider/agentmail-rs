use crate::client::NoBody;
use crate::client::scope::{InboxScope, Scoped};
use crate::{Error, Page, types::*, util::urlish};

/// The inbox's calendar. Every calendar endpoint is inbox-scoped; writes take
/// an optional `etag` (from a prior read) for optimistic concurrency, and the
/// API answers 428 when a newer revision exists.
impl Scoped<'_, InboxScope<'_>> {
    /// GET `{base}/calendar`, the inbox's calendar settings.
    pub async fn get_calendar(&self, consistency: Option<Consistency>) -> Result<Calendar, Error> {
        let query = consistency
            .map(|c| vec![("consistency", c.as_str().to_string())])
            .unwrap_or_default();
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/calendar", self.base()),
                &query,
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{base}/calendar`, move the calendar to another IANA timezone.
    /// Pass the `etag` from a prior read to reject concurrent edits.
    pub async fn update_calendar(
        &self,
        timezone: &str,
        etag: Option<&str>,
    ) -> Result<Calendar, Error> {
        self.client
            .request_with_headers(
                reqwest::Method::PATCH,
                &format!("{}/calendar", self.base()),
                &[],
                Some(&serde_json::json!({ "timezone": timezone })),
                &Self::if_match(etag),
            )
            .await
    }

    /// GET `{base}/calendar/agenda`, occurrences in a window, flattened:
    /// recurring series appear as their individual instances.
    pub async fn get_agenda(&self, query: CalendarEventsQuery) -> Result<CalendarEventList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/calendar/agenda", self.base()),
                &query.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every occurrence in the query's window, draining pagination.
    pub async fn list_all_agenda(
        &self,
        query: CalendarEventsQuery,
    ) -> Result<Vec<CalendarEvent>, Error> {
        let mut out = Vec::new();
        let mut token = query.page_token.clone();
        loop {
            let page = token.clone();
            let resp = self
                .get_agenda(CalendarEventsQuery {
                    page_token: page.clone(),
                    limit: None,
                    ..query.clone()
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.events);
            // A server that keeps returning the same cursor would spin here
            // forever; stop instead.
            if next.is_some() && next == page {
                return Ok(out);
            }
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// GET `{base}/calendar/events`, one page of the calendar's stored events
    /// (series stay unexpanded; use the agenda for flattened occurrences).
    pub async fn list_calendar_events(&self, page: Page) -> Result<CalendarEventList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/calendar/events", self.base()),
                &page.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every stored event, draining pagination.
    pub async fn list_all_calendar_events(&self) -> Result<Vec<CalendarEvent>, Error> {
        let mut out = Vec::new();
        let mut token = None;
        loop {
            let resp = self
                .list_calendar_events(Page {
                    limit: None,
                    page_token: token,
                })
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.events);
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// POST `{base}/calendar/events`, create an event (or series, with a
    /// `recurrence` rule).
    pub async fn create_calendar_event(
        &self,
        event: CreateCalendarEvent,
    ) -> Result<CalendarEventMutation, Error> {
        self.client
            .request(
                reqwest::Method::POST,
                &format!("{}/calendar/events", self.base()),
                &[],
                Some(&event),
            )
            .await
    }

    /// GET `{base}/calendar/events/{event_id}`.
    pub async fn get_calendar_event(
        &self,
        event_id: &str,
        consistency: Option<Consistency>,
    ) -> Result<CalendarEvent, Error> {
        let query = consistency
            .map(|c| vec![("consistency", c.as_str().to_string())])
            .unwrap_or_default();
        self.client
            .request(
                reqwest::Method::GET,
                &format!("{}/calendar/events/{}", self.base(), urlish(event_id)),
                &query,
                None::<&NoBody>,
            )
            .await
    }

    /// PATCH `{base}/calendar/events/{event_id}`. On a series instance, pass
    /// `mode` to move just that occurrence or it and every later one.
    pub async fn update_calendar_event(
        &self,
        event_id: &str,
        update: UpdateCalendarEvent,
        mode: Option<MutationMode>,
        etag: Option<&str>,
    ) -> Result<CalendarEventMutation, Error> {
        let mut query = Vec::new();
        if let Some(mode) = mode {
            query.push(("mode", mode.as_str().to_string()));
        }
        self.client
            .request_with_headers(
                reqwest::Method::PATCH,
                &format!("{}/calendar/events/{}", self.base(), urlish(event_id)),
                &query,
                Some(&update),
                &Self::if_match(etag),
            )
            .await
    }

    /// DELETE `{base}/calendar/events/{event_id}`. On a series instance,
    /// `mode` picks between cancelling just that occurrence and every future
    /// one.
    pub async fn delete_calendar_event(
        &self,
        event_id: &str,
        mode: Option<MutationMode>,
        send_invites: Option<bool>,
        etag: Option<&str>,
    ) -> Result<DeleteCalendarEventResult, Error> {
        let mut query = Vec::new();
        if let Some(mode) = mode {
            query.push(("mode", mode.as_str().to_string()));
        }
        if let Some(send) = send_invites {
            query.push(("send_invites", send.to_string()));
        }
        self.client
            .request_with_headers(
                reqwest::Method::DELETE,
                &format!("{}/calendar/events/{}", self.base(), urlish(event_id)),
                &query,
                None::<&NoBody>,
                &Self::if_match(etag),
            )
            .await
    }

    /// GET `{base}/calendar/events/{event_id}/instances`, the occurrences of
    /// a recurring event in a window.
    pub async fn list_calendar_event_instances(
        &self,
        event_id: &str,
        query: CalendarEventsQuery,
    ) -> Result<CalendarEventList, Error> {
        self.client
            .request(
                reqwest::Method::GET,
                &format!(
                    "{}/calendar/events/{}/instances",
                    self.base(),
                    urlish(event_id)
                ),
                &query.query(),
                None::<&NoBody>,
            )
            .await
    }

    /// Every instance of a recurring event in the query's window, draining
    /// pagination.
    pub async fn list_all_calendar_event_instances(
        &self,
        event_id: &str,
        query: CalendarEventsQuery,
    ) -> Result<Vec<CalendarEvent>, Error> {
        let mut out = Vec::new();
        let mut token = query.page_token.clone();
        loop {
            let page = token.clone();
            let resp = self
                .list_calendar_event_instances(
                    event_id,
                    CalendarEventsQuery {
                        page_token: page.clone(),
                        limit: None,
                        ..query.clone()
                    },
                )
                .await?;
            let next = resp.next_page_token;
            out.extend(resp.events);
            // Same non-advancing-cursor guard as `list_all_agenda`.
            if next.is_some() && next == page {
                return Ok(out);
            }
            match next {
                Some(t) => token = Some(t),
                None => return Ok(out),
            }
        }
    }

    /// POST `{base}/calendar/events/{event_id}/respond`, reply to an invite
    /// on behalf of this inbox.
    pub async fn respond_to_calendar_event(
        &self,
        event_id: &str,
        respond: RespondToCalendarEvent,
        etag: Option<&str>,
    ) -> Result<CalendarEventMutation, Error> {
        self.client
            .request_with_headers(
                reqwest::Method::POST,
                &format!(
                    "{}/calendar/events/{}/respond",
                    self.base(),
                    urlish(event_id)
                ),
                &[],
                Some(&respond),
                &Self::if_match(etag),
            )
            .await
    }

    fn if_match(etag: Option<&str>) -> Vec<(&'static str, String)> {
        etag.map(|e| vec![("If-Match", e.to_string())])
            .unwrap_or_default()
    }
}
