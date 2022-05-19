use crate::types::{Account, CaptchaTokenResponse};
use futures::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, BasicPublishOptions, QueueBindOptions},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use reqwest::StatusCode;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::{oneshot, Mutex};

pub mod types;

const HOSTNAME: &str = "amqp.autosolve.aycd.io";
const VHOST: &str = "oneclick";
const DIRECT_EXCHANGE_PREFIX: &str = "exchanges.direct";
const FANOUT_EXCHANGE_PREFIX: &str = "exchanges.fanout";

const RESPONSE_QUEUE_PREFIX: &str = "queues.response.direct";

const REQUEST_TOKEN_ROUTE_PREFIX: &str = "routes.request.token";
#[allow(dead_code)]
const REQUEST_TOKEN_CANCEL_ROUTE_PREFIX: &str = "routes.request.token.cancel";
const RESPONSE_TOKEN_ROUTE_PREFIX: &str = "routes.response.token";
const RESPONSE_TOKEN_CANCEL_ROUTE_PREFIX: &str = "routes.response.token.cancel";

const AUTO_ACK_QUEUE: bool = true;
const EXCLUSIVE_QUEUE: bool = true;

#[allow(dead_code)]
#[derive(Clone)]
pub struct Client {
    account: Account,
    client_id: String,

    connection: Arc<Connection>,
    direct_channel: Channel,
    fanout_channel: Channel,

    direct_exchange_name: String,
    fanout_exchange_name: String,
    request_token_route_key: String,

    pending_requests: Arc<Mutex<HashMap<String, Option<oneshot::Sender<CaptchaTokenResponse>>>>>,
}

impl Client {
    pub async fn connect(
        client_id: &str,
        access_token: &str,
        api_key: &str,
    ) -> Result<Self, types::Error> {
        if api_key.is_empty() {
            return Err("invalid api key".into());
        } else if access_token.is_empty() {
            return Err("invalid access token".into());
        }

        let token_parts = access_token.split('-').collect::<Vec<_>>();
        if token_parts.is_empty() {
            return Err("invalid access token".into());
        }

        let account_id = i64::from_str(token_parts.get(0).unwrap())?;
        let account = Account {
            id: account_id,
            raw_id: account_id.to_string(),
            access_token: access_token.to_string(),
            raw_access_token: access_token.replace('-', ""),
            api_key: api_key.to_string(),
            raw_api_key: api_key.replace('-', ""),
        };

        let is_valid_account = Self::verify_credentials(client_id, &account).await?;
        if !is_valid_account {
            return Err("invalid account credentials".into());
        }

        let direct_exchange_name = Self::create_key_with_account(&account, DIRECT_EXCHANGE_PREFIX);
        let fanout_exchange_name = Self::create_key_with_account(&account, FANOUT_EXCHANGE_PREFIX);

        let response_queue_name =
            Self::create_key_with_account_and_api(&account, RESPONSE_QUEUE_PREFIX);
        let response_token_route_key =
            Self::create_key_with_account_and_api(&account, RESPONSE_TOKEN_ROUTE_PREFIX);
        let response_token_cancel_route_key =
            Self::create_key_with_account_and_api(&account, RESPONSE_TOKEN_CANCEL_ROUTE_PREFIX);

        let request_token_route_key =
            Self::create_key_with_access_token(&account, REQUEST_TOKEN_ROUTE_PREFIX);
        // let request_token_cancel_route_key =
        //     Self::create_key_with_access_token(&account, REQUEST_TOKEN_CANCEL_ROUTE_PREFIX);

        let amqp_uri = format!(
            "amqp://{}:{}@{}:5672/{}?heartbeat=10",
            account.raw_id, account.access_token, HOSTNAME, VHOST
        );
        let connection = Connection::connect(&amqp_uri, ConnectionProperties::default()).await?;
        let direct_channel = connection.create_channel().await?;
        let fanout_channel = connection.create_channel().await?;

        direct_channel
            .queue_bind(
                &response_queue_name,
                &direct_exchange_name,
                &response_token_route_key,
                QueueBindOptions::default(),
                Default::default(),
            )
            .await?;
        direct_channel
            .queue_bind(
                &response_queue_name,
                &direct_exchange_name,
                &response_token_cancel_route_key,
                QueueBindOptions::default(),
                Default::default(),
            )
            .await?;

        let pending_requests = Arc::new(Mutex::new(HashMap::<
            String,
            Option<oneshot::Sender<CaptchaTokenResponse>>,
        >::new()));
        let pending_requests_listener = pending_requests.clone();

        let consume_opts = BasicConsumeOptions {
            no_ack: AUTO_ACK_QUEUE,
            exclusive: EXCLUSIVE_QUEUE,
            ..Default::default()
        };
        let mut consumer = direct_channel
            .basic_consume(&response_queue_name, "", consume_opts, Default::default())
            .await?;
        tokio::spawn(async move {
            while let Some(Ok(delivery)) = consumer.next().await {
                match delivery.routing_key.as_str() {
                    key if key == response_token_route_key => {
                        match serde_json::from_slice::<CaptchaTokenResponse>(&delivery.data) {
                            Ok(resp) => {
                                let mut pending_requests = pending_requests_listener.lock().await;
                                if let Some(c) = pending_requests
                                    .get_mut(&resp.task_id)
                                    .and_then(|c| c.take())
                                {
                                    let _ = c.send(resp);
                                }
                            }
                            Err(e) => println!("error parsing token resp: {}", e),
                        }
                    }
                    key if key == response_token_cancel_route_key => {
                        println!("received token cancel response");
                    }
                    key => {
                        println!("received unknown response: {}", key);
                    }
                }
            }
            println!("done");
        });

        Ok(Self {
            account,
            client_id: client_id.to_string(),
            connection: Arc::new(connection),
            direct_channel,
            fanout_channel,
            direct_exchange_name,
            fanout_exchange_name,
            request_token_route_key,
            pending_requests,
        })
    }

    pub async fn send_token_request(
        &self,
        mut req: types::CaptchaTokenRequest,
    ) -> Result<oneshot::Receiver<CaptchaTokenResponse>, types::Error> {
        req.api_key = self.account.api_key.clone();
        req.created_at = shared::util::epoch_time().as_secs();
        self.direct_channel
            .basic_publish(
                &self.direct_exchange_name,
                &self.request_token_route_key,
                BasicPublishOptions::default(),
                &serde_json::to_vec(&req)?,
                BasicProperties::default().with_content_encoding("application/json".into()),
            )
            .await?;
        let (tx, rx) = oneshot::channel();
        let mut pending_requests = self.pending_requests.lock().await;
        pending_requests.insert(req.task_id, Some(tx));
        Ok(rx)
    }

    // TODO: check response code for more specific errors
    async fn verify_credentials(client_id: &str, account: &Account) -> Result<bool, types::Error> {
        let resp = reqwest::get(format!(
            "https://dash.autosolve.aycd.io/rest/{}/verify/{}?clientId={}",
            account.access_token, account.api_key, client_id
        ))
        .await?;
        Ok(resp.status() == StatusCode::OK)
    }

    fn create_key_with_access_token(account: &Account, prefix: &str) -> String {
        format!("{}.{}", prefix, account.raw_access_token)
    }

    fn create_key_with_account(account: &Account, prefix: &str) -> String {
        format!("{}.{}", prefix, account.raw_id)
    }

    fn create_key_with_account_and_api(account: &Account, prefix: &str) -> String {
        format!("{}.{}.{}", prefix, account.raw_id, account.raw_api_key)
    }
}

#[cfg(test)]
mod tests {
    use crate::{types::CaptchaTokenRequest, Client};
    use std::time::Duration;

    const CLIENT_ID: &str = "Nebula-c92504a1-5441-4970-9218-be520bc5416c";
    const API_KEY: &str = "70ebaf60-5cd9-4aaa-8983-301cee5fe983";
    const ACCESS_TOKEN: &str = "19272-8ee8281c-fd79-4685-a9c0-04edf07bee2d";

    #[tokio::test]
    async fn connect() {
        let client = Client::connect(CLIENT_ID, ACCESS_TOKEN, API_KEY).await;
        assert!(client.is_ok());
        let client = client.unwrap();
        let token_res = client
            .send_token_request(CaptchaTokenRequest {
                task_id: "test-task".to_string(),
                url: "https://www.yeezysupply.com/".to_string(),
                site_key: "6Lf34M8ZAAAAANgE72rhfideXH21Lab333mdd2d-".into(),
                version: 2,
                min_score: 0.1,
                action: "yzysply_wr_pageview".into(),
                ..Default::default()
            })
            .await;
        assert!(token_res.is_ok());
        let token_chan = token_res.unwrap();
        let token_resp = token_chan.await.unwrap();
        dbg!(token_resp);
    }
}
