use crate::{
    flashbots::{BundleRequest, FlashbotsMiddleware, PendingBundleError},
    Credentials,
};
use ethers::prelude::*;
use log::*;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{RequestBuilder, Response};
use shared::config::Config as NftyConfig;
use std::{io::Cursor, sync::Arc, time::Duration};

pub trait StaticMiddleware = 'static + Middleware;
pub trait StaticSigner = 'static + Signer + Clone;

pub static SOUND_FILE: &[u8] = include_bytes!("../assets/success.mp3");

#[derive(Clone)]
pub struct Context<M, S> {
    session_id: String,
    credentials: Credentials,
    config: NftyConfig,
    http: reqwest::Client,
    autosolve: Option<autosolve::Client>,
    provider: Arc<SignerMiddleware<FlashbotsMiddleware<M, S>, S>>,
}

impl<M: StaticMiddleware, S: StaticSigner> Context<M, S> {
    pub async fn new(
        config: NftyConfig,
        provider: SignerMiddleware<FlashbotsMiddleware<M, S>, S>,
        autosolve: Option<autosolve::Client>,
        credentials: Credentials,
        session_id: String,
    ) -> Result<Self, shared::Error> {
        let http = config.create_http_client()?;

        Ok(Self {
            credentials,
            session_id,
            config,
            http,
            autosolve,
            provider: Arc::new(provider),
        })
    }

    pub fn session(&self) -> &str {
        self.session_id.as_str()
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    pub fn config(&self) -> &NftyConfig {
        &self.config
    }

    pub fn provider(&self) -> &SignerMiddleware<FlashbotsMiddleware<M, S>, S> {
        &self.provider
    }

    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub fn autosolve(&self) -> Option<&autosolve::Client> {
        self.autosolve.as_ref()
    }

    pub async fn delay(&self, message: &str) {
        if let Some(os) = &self.config.opensea {
            let delay = os.api_delay.unwrap_or(500);
            info!("{}, sleeping for {}ms...", message, delay);
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
    }

    pub async fn delay_warn(&self, message: &str) {
        if let Some(os) = &self.config.opensea {
            let delay = os.api_delay.unwrap_or(500);
            warn!("{}, sleeping for {}ms...", message, delay);
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }
    }

    pub async fn handle_os_request(
        &self,
        builder: RequestBuilder,
    ) -> Result<Response, shared::Error> {
        let origin: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect();

        let builder = if let Some(api_key) = self
            .config()
            .opensea
            .as_ref()
            .expect("expected OpenSea config")
            .api_key
            .as_ref()
        {
            builder.header("x-api-key", api_key)
        } else {
            builder
        };

        Ok(builder.header("origin", origin).send().await?)
    }

    async fn play_success_sound() {
        /*
            let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
            let sink = rodio::Sink::try_new(&handle).unwrap();

            let file = std::fs::File::open("success.mp3").unwrap();
            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());

            sink.sleep_until_end();
        */

        if let Ok((_stream, handle)) = rodio::OutputStream::try_default() {
            if let Ok(sink) = rodio::Sink::try_new(&handle) {
                let buf = Cursor::new(SOUND_FILE);
                sink.append(rodio::Decoder::new(buf).unwrap());
                sink.sleep_until_end();
            }
        }
    }

    pub async fn send_bundle(&self, bundle: &BundleRequest) -> Result<(), shared::Error> {
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

        info!("Sending bundle...");
        let pending_bundle = self.provider().inner().send_bundle(bundle).await?;
        match pending_bundle.await {
            Ok(_) => {
                info!("Bundle included!");
                tokio::spawn(Self::play_success_sound());
                Ok(())
            }
            Err(PendingBundleError::BundleNotIncluded) => {
                warn!("Bundle was not included in target block!");
                Err(PendingBundleError::BundleNotIncluded.into())
            }
            Err(e) => {
                error!("Error sending bundle: {}", e);
                Err(e.into())
            }
        }
    }
}
