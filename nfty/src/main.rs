#![feature(trait_alias)]
#![feature(option_result_contains)]

use crate::flashbots::FlashbotsMiddleware;
use argh::FromArgs;
use ethers::prelude::*;
use log::*;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use shared::config::{Config as NftyConfig, Mode, OSAPI};
use std::{error::Error as StdError, str::FromStr, time::Duration};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

pub use context::*;

pub mod util;

mod context;
pub mod flashbots;
mod looksrare;
mod mint;
mod model;
mod opensea;
mod themida;

pub type Error = Box<dyn StdError + Send + Sync>;

#[derive(FromArgs)]
/// opensea goes brrrr
struct App {
    /// optional path to config file
    #[argh(option, short = 'c')]
    config: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Credentials {
    token_id: Option<u64>,
    owner_key: String,
    autosolve_api_key: Option<String>,
    autosolve_access_token: Option<String>,
}

#[derive(Debug, Serialize)]
struct VerifyRequest {
    id: Uuid,
    token: u64,
    address: String,
    signature: String,
}

#[derive(Debug, Deserialize)]
struct VerifyResponse {
    session_id: Uuid,
    idempotency: String,
    identifier: u64,
}

#[derive(Debug, Serialize)]
struct HeartbeatRequest {
    id: Uuid,
    token: u64,
    address: String,
    signature: String,
}

#[derive(Debug, Deserialize)]
struct HeartbeatResponse {
    idempotency: String,
    identifier: u64,
}

lazy_static::lazy_static! {
    pub static ref HAS_AUTHED: Mutex<i32> = Mutex::new(i32::MAX);
}

#[inline(always)]
async fn auth(credentials: &Credentials) -> Result<String, Error> {
    let c = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .no_proxy()
        .build()?;

    const EXPIRATION_TIME: u64 = 1648616094;

    #[cfg(feature = "themida")]
    unsafe {
        themida::STR_ENCRYPT_START()
    }

    let akamai_time = u64::from_str(
        &c.get("https://time.akamai.com/")
            .send()
            .await
            .expect("0.01")
            .text()
            .await
            .expect("0.02"),
    )
    .expect("0.03");

    #[cfg(feature = "themida")]
    unsafe {
        themida::STR_ENCRYPT_END()
    }

    #[cfg(feature = "themida")]
    unsafe {
        themida::VM_EAGLE_BLACK_START()
    }

    let current_time = shared::util::epoch_time().as_secs();

    if current_time > EXPIRATION_TIME || akamai_time > EXPIRATION_TIME {
        std::process::exit(-1);
    }

    #[cfg(feature = "themida")]
    unsafe {
        themida::VM_EAGLE_BLACK_END()
    }

    let id = Uuid::new_v4();
    let token = credentials.token_id.unwrap_or(0);
    let signer = Wallet::from_str(&credentials.owner_key)?;
    let signature = signer.sign_message(id.as_bytes().to_vec()).await?;

    let auth_resp = c
        .post("https://nfty-api.721.gg/v1/verify")
        .json(&VerifyRequest {
            id,
            token,
            address: format!("0x{:x}", signer.address()),
            signature: signature.to_string(),
        })
        .send()
        .await
        .expect("0x00");
    match auth_resp.status() {
        StatusCode::OK => {}
        StatusCode::PAYMENT_REQUIRED => panic!("Renew your token to continue using NFTYBot."),
        status => panic!("Unexpected auth status: {}", status),
    };
    let auth_resp = auth_resp
        .error_for_status()
        .expect("0x01")
        .json::<VerifyResponse>()
        .await
        .expect("0x02");

    let idempotency = hex::decode(auth_resp.idempotency)?;
    let timestamp = (u128::from_be_bytes(idempotency.as_slice().try_into().expect("0x03")) >> 64)
        - auth_resp.identifier as u128;
    let current_time = shared::util::epoch_time().as_secs() as u128;
    if timestamp >= current_time + 15 || timestamp <= current_time - 15 {
        std::process::exit(-1);
    }

    if *HAS_AUTHED.lock().await != i32::MAX {
        std::process::exit(-1);
    }
    *HAS_AUTHED.lock().await = 6969;

    let session_id = auth_resp.session_id;

    tokio::spawn(async move {
        let mut failed_heartbeats = 0;
        loop {
            if failed_heartbeats >= 3 {
                std::process::exit(-1);
            }
            tokio::time::sleep(Duration::from_secs(15)).await;

            let signature = signer
                .sign_message(session_id.as_bytes().to_vec())
                .await
                .unwrap();

            let heartbeat_resp = c
                .post("https://nfty-api.721.gg/v1/heartbeat")
                .json(&HeartbeatRequest {
                    id: auth_resp.session_id,
                    token,
                    address: format!("0x{:x}", signer.address()),
                    signature: signature.to_string(),
                })
                .send()
                .await;
            let heartbeat_resp = match heartbeat_resp.and_then(|r| r.error_for_status()) {
                Ok(heartbeat_resp) => match heartbeat_resp.json::<HeartbeatResponse>().await {
                    Ok(resp) => resp,
                    _ => {
                        failed_heartbeats += 1;
                        continue;
                    }
                },
                _ => {
                    failed_heartbeats += 1;
                    continue;
                }
            };

            failed_heartbeats = 0;

            let idempotency = hex::decode(heartbeat_resp.idempotency).unwrap();
            let timestamp = (u128::from_be_bytes(idempotency.as_slice().try_into().expect("0x03"))
                >> 64)
                - heartbeat_resp.identifier as u128;
            let current_time = shared::util::epoch_time().as_secs() as u128;
            if timestamp >= current_time + 15 || timestamp <= current_time - 15 {
                std::process::exit(-1);
            }

            if *HAS_AUTHED.lock().await != 6969 {
                std::process::exit(-1);
            }
        }
    });

    Ok(auth_resp.session_id.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _ = dotenv::dotenv();
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let credentials = toml::from_slice::<Credentials>(
        &std::fs::read("credentials.toml").expect("Missing credentials.toml file"),
    )?;
    let session_id = auth(&credentials).await?;

    pretty_env_logger::init_timed();
    let app: App = argh::from_env();

    println!("test");

    let configs = if app.config.is_empty() {
        vec!["config.toml".to_string()]
    } else {
        app.config
    };

    info!("using config(s): {}", &configs.join(", "));

    let mut futs = Vec::with_capacity(configs.len());

    const CLIENT_ID: &str = "Nebula-c92504a1-5441-4970-9218-be520bc5416c";
    let autosolve: Option<autosolve::Client> = match (
        credentials.autosolve_api_key.as_ref(),
        credentials.autosolve_access_token.as_ref(),
    ) {
        (Some(api_key), Some(access_token)) => {
            Some(autosolve::Client::connect(CLIENT_ID, access_token, api_key).await?)
        }
        _ => None,
    };

    for path in configs {
        let config = toml::from_slice::<NftyConfig>(&tokio::fs::read(&path).await?)?;

        let ws = Ws::connect(&config.global.provider_url).await?;
        let base_provider = Provider::<Ws>::new(ws).interval(Duration::from_millis(1000));

        let wallet = Wallet::from_str(&config.account.private_key)?;
        let our_addr = wallet.address();
        let our_bal = base_provider.get_balance(our_addr, None).await?;
        info!(
            "using account: 0x{:x}, balance: {}",
            our_addr,
            if our_bal > U256::from(f64::MAX as u128) {
                -1.
            } else {
                our_bal.as_u128() as f64 / 1e18
            }
        );

        if config.global.relays.is_empty() {
            panic!("must have at least 1 flashbots relay");
        }

        let flashbots_signer = match config.global.flashbots_signer.as_ref() {
            Some(signer) if !signer.is_empty() => Wallet::from_str(signer)?,
            _ => wallet.clone(),
        };
        let provider = SignerMiddleware::new(
            FlashbotsMiddleware::new(
                base_provider,
                config
                    .global
                    .relays
                    .iter()
                    .map(|u| Url::parse(u).unwrap())
                    .collect(),
                flashbots_signer,
            ),
            wallet,
        );

        let ctx = Context::new(
            config,
            provider,
            autosolve.clone(),
            credentials.clone(),
            session_id.clone(),
        )
        .await?;
        futs.push(tokio::spawn(async move {
            info!("starting task for: {}", &path);
            launch(ctx, our_addr).await
        }));
    }

    for fut in futs {
        match fut.await {
            Ok(Ok(())) => {}
            Ok(Err(why)) => error!("error awaiting task: {:?}", why),
            Err(why) => error!("error awaiting task: {:?}", why),
        }
    }

    Ok(())
}

async fn launch<M: StaticMiddleware + Clone, S: StaticSigner>(
    ctx: Context<M, S>,
    our_addr: Address,
) -> Result<(), Error> {
    if let Ok(stats) = ctx.provider().inner().get_user_stats().await {
        let total: f64 = ethers::utils::format_units(stats.all_time_miner_payments, 15)
            .as_u32()
            .into();

        let spent = total / 1000.0;

        info!(
            "Flashbots stats: high_priority={}, miner_reward_sent={}",
            stats.is_high_priority, spent,
        );
    } else {
        warn!("Missing flashbots data")
    }

    #[cfg(feature = "themida")]
    unsafe {
        crate::themida::VM_DOLPHIN_BLACK_START()
    }

    if *crate::HAS_AUTHED.lock().await != 6969 {
        std::process::exit(-1);
    }

    #[cfg(feature = "themida")]
    unsafe {
        crate::themida::VM_DOLPHIN_BLACK_END()
    }

    // retard protection
    if ctx.config().global.mode == Mode::Mint {
        if let Some(mint_config) = ctx.config().mint.as_ref() {
            if !ctx.config().account.dry_run
                && mint_config.contract_address == *"0x0000000000000000000000000000000000000000"
            {
                panic!("stop. get some help.");
            }
        }
    }

    match ctx.config().global.mode {
        // Mode::Drop => opensea::modules::drop::handle(&ctx, our_addr).await?,
        Mode::Mint => mint::handle(ctx, our_addr).await?,
        Mode::OpenSeaLimit => match ctx
            .config()
            .opensea
            .as_ref()
            .expect("expected OpenSea config")
            .api
        {
            OSAPI::Rest => opensea::modules::limit::rest::handle(&ctx, our_addr).await?,
            OSAPI::GraphQL => opensea::modules::limit::gql::handle(&ctx, our_addr).await?,
        },
        Mode::LooksRareLimit => looksrare::modules::limit::handle(&ctx, our_addr).await?,
        _ => {}
    };

    Ok(())
}
