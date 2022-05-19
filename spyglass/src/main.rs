use ethers::prelude::*;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::standard::{
        macros::{command, group},
        Args, CommandResult, StandardFramework,
    },
    model::{channel::Message, id::ChannelId, prelude::Ready},
    prelude::TypeMapKey,
    utils::Color,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error as StdError,
    ops::Div,
    str::FromStr,
    sync::Arc,
};
use tokio::{sync::Mutex, time::Duration};

pub struct PendingContainer;

impl TypeMapKey for PendingContainer {
    type Value = Arc<Mutex<HashMap<Address, HashSet<TxHash>>>>;
}

pub struct WatchlistContainer;

impl TypeMapKey for WatchlistContainer {
    type Value = Arc<Mutex<HashSet<Address>>>;
}

pub struct ProviderContainer;

impl TypeMapKey for ProviderContainer {
    type Value = Arc<Provider<Ws>>;
}

#[group]
#[commands(watch, forgor)]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);

        let provider = {
            let data = ctx.data.read().await;
            data.get::<ProviderContainer>().unwrap().clone()
        };

        #[derive(Clone)]
        struct TxData {
            hash: H256,
            to: Address,
            from: Address,
            value: U256,
            block_number: String,
            formatted_gas: String,
            method_id: String,
            pending: usize,
        }

        let shared_tx_queue = Arc::new(Mutex::new(Vec::new()));

        let tx_ctx = ctx.clone();
        let tx_provider = provider.clone();
        let tx_queue = shared_tx_queue.clone();
        tokio::spawn(async move {
            let mut tx_stream = tx_provider.subscribe_pending_txs().await.unwrap();
            while let Some(tx) = tx_stream.next().await {
                if let Ok(Some(tx)) = tx_provider.get_transaction(tx).await {
                    if let Some(to_addr) = tx.to {
                        let is_watched = {
                            let data = tx_ctx.data.read().await;
                            let watchlist = data.get::<WatchlistContainer>().unwrap().lock().await;
                            watchlist.contains(&to_addr)
                        };
                        if !is_watched {
                            continue;
                        }

                        let pending_txs = {
                            let data = tx_ctx.data.read().await;
                            let mut pending = data.get::<PendingContainer>().unwrap().lock().await;
                            match pending.get_mut(tx.to.as_ref().unwrap()) {
                                Some(a) => {
                                    a.insert(tx.hash);
                                    a.len()
                                }
                                None => {
                                    let mut set = HashSet::new();
                                    set.insert(tx.hash);
                                    pending.insert(tx.to.unwrap(), set);
                                    1
                                }
                            }
                        };

                        let data = tx.input.to_vec();

                        let tx_data = TxData {
                            hash: tx.hash,
                            to: to_addr,
                            from: tx.from,
                            value: tx.value,
                            block_number: "Pending".into(),
                            formatted_gas: match (
                                tx.gas_price,
                                tx.max_fee_per_gas,
                                tx.max_priority_fee_per_gas,
                            ) {
                                (_, Some(max_per), Some(max_priority)) => {
                                    format!(
                                        "{} / {} Gwei",
                                        max_per.div(1e9 as u64),
                                        max_priority.div(1e9 as u64)
                                    )
                                }
                                (Some(gas), _, _) => {
                                    format!("{} Gwei", gas.div(1e9 as u64))
                                }
                                _ => "Unknown".into(),
                            },
                            method_id: if data.len() >= 4 {
                                hex::encode(&data[0..4])
                            } else {
                                "None".to_string()
                            },
                            pending: pending_txs,
                        };
                        tx_queue.lock().await.push(tx_data);
                    }
                }
            }
        });

        let tx_ctx = ctx.clone();
        let tx_queue = shared_tx_queue;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;

                let txs = {
                    let mut txs = tx_queue.lock().await;
                    let ret_txs = txs.to_vec();
                    *txs = Vec::new();
                    ret_txs
                };
                if txs.is_empty() {
                    continue;
                }

                for tx_chunk in txs.chunks(10) {
                    let _ = ChannelId(890177121111134208)
                        .send_message(&tx_ctx.http, |m| {
                            for tx in tx_chunk {
                                m.add_embed(|e| {
                                    e.title("Transaction Pending")
                                        .field("To", format!("0x{:x}", tx.to), false)
                                        .field("From", format!("0x{:x}", tx.from), false)
                                        .field(
                                            "Value",
                                            format!("{} ETH", tx.value.as_u128() as f64 / 1e18),
                                            true,
                                        )
                                        .field("Gas Fee", &tx.formatted_gas, true)
                                        .field("Method", &tx.method_id, true)
                                        .field("Block", &tx.block_number, true)
                                        .field("Pending", tx.pending, true)
                                        .field("Hash", format!("0x{:x}", tx.hash), false)
                                        .color(Color::GOLD)
                                });
                            }
                            m
                        })
                        .await;
                }
            }
        });

        let block_ctx = ctx.clone();
        let block_provider = provider;
        tokio::spawn(async move {
            let mut block_stream = block_provider.subscribe_blocks().await.unwrap();
            while let Some(block) = block_stream.next().await {
                if let Ok(Some(block)) = block_provider.get_block(block.hash.unwrap()).await {
                    let mut txs = Vec::new();
                    for tx in block.transactions {
                        if let Ok(Some(tx)) = block_provider.get_transaction(tx).await {
                            if let Some(to_addr) = tx.to {
                                let is_watched = {
                                    let data = block_ctx.data.read().await;
                                    let watchlist =
                                        data.get::<WatchlistContainer>().unwrap().lock().await;
                                    watchlist.contains(&to_addr)
                                };
                                if !is_watched {
                                    continue;
                                }

                                let pending_txs = {
                                    let data = block_ctx.data.read().await;
                                    let mut pending =
                                        data.get::<PendingContainer>().unwrap().lock().await;
                                    match pending.get_mut(tx.to.as_ref().unwrap()) {
                                        Some(a) => {
                                            if a.get(&tx.hash).is_some() {
                                                a.remove(&tx.hash);
                                                a.len()
                                            } else {
                                                0
                                            }
                                        }
                                        None => 0,
                                    }
                                };

                                let data = tx.input.to_vec();

                                let tx_data = TxData {
                                    hash: tx.hash,
                                    to: to_addr,
                                    from: tx.from,
                                    value: tx.value,
                                    block_number: tx.block_number.unwrap().to_string(),
                                    formatted_gas: match (
                                        tx.gas_price,
                                        tx.max_fee_per_gas,
                                        tx.max_priority_fee_per_gas,
                                    ) {
                                        (_, Some(max_per), Some(max_priority)) => {
                                            format!(
                                                "{} / {} Gwei",
                                                max_per.div(1e9 as u64),
                                                max_priority.div(1e9 as u64)
                                            )
                                        }
                                        (Some(gas), _, _) => {
                                            format!("{} Gwei", gas.div(1e9 as u64))
                                        }
                                        _ => "Unknown".into(),
                                    },
                                    method_id: if data.len() >= 4 {
                                        hex::encode(&data[0..4])
                                    } else {
                                        "None".to_string()
                                    },
                                    pending: pending_txs,
                                };
                                txs.push(tx_data);
                            }
                        }
                    }

                    for tx_chunk in txs.chunks(10) {
                        let _ = ChannelId(890177121111134208)
                            .send_message(&block_ctx.http, |m| {
                                for tx in tx_chunk {
                                    m.add_embed(|e| {
                                        e.title("Transaction Mined")
                                            .field("To", format!("0x{:x}", tx.to), false)
                                            .field("From", format!("0x{:x}", tx.from), false)
                                            .field(
                                                "Value",
                                                format!("{} ETH", tx.value.as_u128() as f64 / 1e18),
                                                true,
                                            )
                                            .field("Gas Fee", &tx.formatted_gas, true)
                                            .field("Method", &tx.method_id, true)
                                            .field("Block", &tx.block_number, true)
                                            .field("Pending", tx.pending.to_string(), true)
                                            .field("Hash", format!("0x{:x}", tx.hash), false)
                                            .color(Color::DARK_GREEN)
                                    });
                                }
                                m
                            })
                            .await;
                    }
                }
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let _ = dotenv::dotenv();

    let ws = Ws::connect("wss://eth-ws.721.gg").await.unwrap();
    let provider = Provider::<Ws>::new(ws).interval(Duration::from_millis(250));

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<PendingContainer>(Arc::new(Mutex::new(HashMap::new())));
        data.insert::<WatchlistContainer>(Arc::new(Mutex::new(HashSet::new())));
        data.insert::<ProviderContainer>(Arc::new(provider));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }

    Ok(())
}

#[command]
async fn watch(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let raw_address = args.single::<String>()?;
    let address = Address::from_str(&raw_address)?;

    {
        let data = ctx.data.read().await;
        let mut watchlist = data.get::<WatchlistContainer>().unwrap().lock().await;
        watchlist.insert(address);
    }

    let provider = {
        let data = ctx.data.read().await;
        data.get::<ProviderContainer>().unwrap().clone()
    };

    let txpool_content = provider.txpool_content().await.unwrap();
    for txs in txpool_content.pending.values() {
        for tx in txs.values() {
            if let Some(to) = &tx.to {
                if *to == address {
                    let data = ctx.data.read().await;
                    let mut pending = data.get::<PendingContainer>().unwrap().lock().await;
                    match pending.get_mut(to) {
                        Some(a) => {
                            a.insert(tx.hash);
                        }
                        None => {
                            let mut set = HashSet::new();
                            set.insert(tx.hash);
                            pending.insert(*to, set);
                        }
                    }
                }
            }
        }
    }

    msg.reply(&ctx, "ok").await?;

    Ok(())
}

#[command]
async fn forgor(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let raw_address = args.single::<String>()?;
    let address = Address::from_str(&raw_address)?;

    {
        let data = ctx.data.read().await;

        let mut watchlist = data.get::<WatchlistContainer>().unwrap().lock().await;
        watchlist.remove(&address);

        let mut pending = data.get::<PendingContainer>().unwrap().lock().await;
        pending.remove(&address);
    }

    msg.reply(&ctx, "ok").await?;

    Ok(())
}
