use crate::NftyConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GET_EVENTS_QUERY: &str = r#"
    query GetEventsQuery($pagination: PaginationInput, $filter: EventFilterInput) {
      events(pagination: $pagination, filter: $filter) {
        ...EventFragment
      }
    }

  fragment EventFragment on Event {
    id
    from
    to
    type
    hash
    createdAt
    token {
      tokenId
      image
      name
    }
    order {
      isOrderAsk
      price
      endTime
      currency
      strategy
      status
      params
    }
  }
"#;

#[derive(Serialize, Deserialize)]
pub struct Query {
    pub id: String,
    pub query: String,
    pub variables: Value,
}

impl Query {
    pub fn new<S: ToString>(id: S, query: S, variables: Value) -> Self {
        Self {
            id: id.to_string(),
            query: query.to_string(),
            variables,
        }
    }
}

pub struct Executor {
    client: reqwest::Client,
}

impl Executor {
    pub fn from_config(config: &NftyConfig) -> Result<Self, shared::Error> {
        let client = config.create_http_client()?;
        Ok(Self::with_client(client))
    }

    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn execute(&self, q: Query) -> Result<reqwest::Response, crate::Error> {
        Ok(self
            .client
            .post("https://api.looksrare.org/graphql")
            .json(&q)
            .send()
            .await?)
    }
}
