use crate::{
    flashbots::BundleRequest,
    opensea,
    opensea::{AssetEvent, AssetEvents, Orders},
    util,
    util::NULL_ADDR,
    Context,
};
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use ethers::prelude::*;
use itertools::Itertools;
use log::*;
use rand::{prelude::*, thread_rng};
use reqwest::StatusCode;
use shared::config::{OSLimitMode, SmartGas};
use tokio::time::Duration;

pub async fn handle<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
) -> Result<(), shared::Error> {
    let opensea_config = ctx
        .config()
        .opensea
        .as_ref()
        .expect("expected OpenSea config");
    let limit_config = opensea_config
        .limit
        .as_ref()
        .expect("expected Limit config");
    match limit_config.mode {
        OSLimitMode::Collection => collection_loop(ctx, our_addr).await,
        OSLimitMode::Token => token_loop(ctx, our_addr).await,
    }
}

async fn collection_loop<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
) -> Result<(), shared::Error> {
    let opensea_config = ctx
        .config()
        .opensea
        .as_ref()
        .expect("expected OpenSea config");
    let limit_config = opensea_config
        .limit
        .as_ref()
        .expect("expected Limit config");

    let collections = limit_config
        .collections
        .as_ref()
        .expect("expected collections");
    if collections.is_empty() {
        panic!("no collections specified");
    }

    let collection = collections.first().unwrap();

    if collections.len() > 1 {
        warn!(
            r#"when api = "Rest" only 1 collection can be monitored at a time, nfty will only monitor the collection: {}"#,
            &collection.slug
        )
    }

    let minimum_price = (collection.minimum_price * 1e18) as u128;
    let maximum_price = (collection.maximum_price * 1e18) as u128;

    let mut last_time = Utc::now();
    loop {
        info!("fetching new listings...");

        let cur_time = Utc::now();
        let mut listings = match fetch_listings(ctx, last_time).await {
            Ok(listings) => listings,
            Err(e) => {
                error!("error fetching new listings: {}", e);
                continue;
            }
        };
        last_time = cur_time;

        if listings.is_empty() {
            ctx.delay("no new listings").await;
            continue;
        }

        listings.shuffle(&mut thread_rng());

        for l in listings
            .into_iter()
            .filter(|l| l.payment_token.symbol == *"ETH" && l.asset.is_some())
        {
            let listing_price = l.starting_price;
            let price = match listing_price.parse::<u128>() {
                Ok(p) => p,
                Err(e) => {
                    error!(
                        "error parsing price for listing, val: {}, error: {}",
                        listing_price, e
                    );
                    continue;
                }
            };
            if price < minimum_price || price > maximum_price {
                continue;
            }

            info!(
                "found potential order matching min/max for token id {} @ {} eth",
                &l.asset.as_ref().unwrap().token_id,
                price as f64 / 1e18
            );

            let orders_resp = fetch_orders(
                ctx,
                l.asset.as_ref().unwrap().asset_contract.address.clone(),
                l.asset.as_ref().unwrap().token_id.clone(),
            )
            .await;

            let orders = match orders_resp {
                Ok(orders) => orders,
                Err(e) => {
                    error!("error fetching orders: {}", e);
                    continue;
                }
            };
            if orders.is_empty() {
                warn!("found no orders for asset, maybe out sniped? ha jk you are using nfty that does not happen.");
                continue;
            }

            for order in orders.into_iter() {
                let base_price = U256::from_dec_str(&order.base_price)?;
                let maximum_price = U256::from(maximum_price);
                if base_price > maximum_price {
                    continue;
                } else if order.listing_time > shared::util::epoch_time().as_secs() {
                    warn!("found order that has listing time in the future, probably an auction. ignoring.");
                    continue;
                }

                info!("found matching order");
                send_tx(ctx, our_addr, maximum_price, base_price, &order).await?;
            }
        }

        ctx.delay("processed new listings").await
    }
}

async fn token_loop<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
) -> Result<(), shared::Error> {
    let opensea_config = ctx
        .config()
        .opensea
        .as_ref()
        .expect("expected OpenSea config");
    let limit_config = opensea_config
        .limit
        .as_ref()
        .expect("expected Limit config");
    let minimum_price =
        U256::from((limit_config.minimum_price.expect("expected minimum_price") * 1e18) as u128);
    let maximum_price =
        U256::from((limit_config.maximum_price.expect("expected maximum_price") * 1e18) as u128);

    loop {
        info!("fetching orders...");

        let orders_resp = fetch_orders(
            ctx,
            limit_config
                .contract_address
                .as_ref()
                .expect("expected contract_address for token limit")
                .clone(),
            limit_config
                .token_id
                .as_ref()
                .expect("expected token_id for token limit")
                .clone(),
        )
        .await;

        let orders = match orders_resp {
            Ok(orders) => orders,
            Err(e) => {
                error!("error fetching orders: {}", e);
                continue;
            }
        };

        let mut filtered_orders = orders
            .into_iter()
            .filter(|o| o.taker.address == *"0x0000000000000000000000000000000000000000")
            .collect_vec();

        if filtered_orders.is_empty() {
            ctx.delay_warn("found no orders for asset").await;
            continue;
        }

        filtered_orders.shuffle(&mut rand::thread_rng());

        for order in filtered_orders {
            let base_price =
                U256::from_dec_str(&order.base_price)? * U256::from_dec_str(&order.quantity)?;
            if base_price > maximum_price || base_price < minimum_price {
                continue;
            } else if order.listing_time >= shared::util::epoch_time().as_secs() {
                warn!("found order that has listing time in the future, probably an auction. ignoring.");
                continue;
            }

            info!(
                "found matching order @ price: {} qty: {}",
                base_price.as_u64() as f64 / 1e18,
                order.quantity
            );
            send_tx(ctx, our_addr, maximum_price, base_price, &order).await?;
        }
        ctx.delay("processed listings").await;
    }
}

async fn send_tx<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
    maximum_price: U256,
    base_price: U256,
    order: &opensea::Order,
) -> Result<(), shared::Error> {
    let opensea_config = ctx
        .config()
        .opensea
        .as_ref()
        .expect("expected OpenSea config");
    let base_gas_fee = U256::from((opensea_config.gas_fee * 1e9) as u128);
    for _ in 0..opensea_config.maximum_retry_attempts {
        let nonce = ctx.provider().get_transaction_count(our_addr, None).await?;

        let gas_fee = match opensea_config.smart_gas {
            SmartGas::Enabled => {
                base_gas_fee.max((maximum_price - base_price) / opensea_config.gas_limit)
            }
            SmartGas::Disabled => base_gas_fee,
            SmartGas::Exclusive => (maximum_price - base_price) / opensea_config.gas_limit,
        };

        let tx = util::new_order_to_tx(
            ctx.config(),
            ctx.provider(),
            our_addr,
            order,
            gas_fee,
            if opensea_config.smart_gas == SmartGas::Exclusive {
                gas_fee
            } else {
                opensea_config
                    .priority_fee
                    .map(|x| U256::from((x * 1e9) as u128))
                    .unwrap_or(gas_fee)
            },
            nonce,
        )
        .await?;
        let signature = ctx.provider().signer().sign_transaction(&tx).await?;

        let mut bundle = BundleRequest::new();
        bundle.push_transaction(tx.rlp_signed(ctx.provider().signer().chain_id(), &signature));

        let block_number = ctx.provider().get_block_number().await?;
        let target_block = block_number + 1;

        bundle
            .set_block(target_block)
            .set_simulation_block(block_number)
            .set_simulation_timestamp(shared::util::epoch_time().as_secs());

        if ctx.config().account.simulate {
            match ctx.provider().inner().simulate_bundle(&bundle).await {
                Ok(simulated_bundle) => {
                    dbg!(
                        target_block,
                        simulated_bundle.effective_gas_price().as_u64() as f64 / 1e9
                    );
                }
                Err(e) => {
                    error!("error simulating bundle: {}", e);
                    break;
                }
            }
        }

        if ctx.config().account.dry_run {
            info!("Dry run, exiting early. Did not send bundle.");
            break;
        }

        if ctx.send_bundle(&bundle).await.is_ok() {
            break;
        }
    }

    Ok(())
}

#[async_recursion]
async fn fetch_listings<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    after_time: DateTime<Utc>,
) -> Result<Vec<AssetEvent>, shared::Error> {
    let opensea_config = ctx
        .config()
        .opensea
        .as_ref()
        .expect("expected OpenSea config");
    let limit_config = opensea_config
        .limit
        .as_ref()
        .expect("expected Limit config");
    let collection = limit_config
        .collections
        .as_ref()
        .expect("expected collections")
        .first()
        .unwrap();
    let mut listings = Vec::new();
    let res = ctx.handle_os_request(
        ctx
            .http()
            .get(
                format!(
                    "https://api.opensea.io/api/v1/events?collection_slug={}&event_type=created&occurred_after={}&only_opensea=true&offset=0&limit=100",
                    &collection.slug,
                    after_time.format("%Y-%m-%dT%T%.6f")
                )
            )
    ).await?;

    // for some reason it seems like once we fetch all assets it just says we're rate limited?
    // so this is a sort of hacky "fix" for that, not really sure how else to handle it atm
    match res.status() {
        StatusCode::OK => {
            let resp = res.text().await?;
            let event_history = match serde_json::from_str::<AssetEvents>(&resp) {
                Ok(event_history) => event_history,
                Err(e) => {
                    dbg!(e, resp);
                    return Ok(listings);
                }
            };
            // let event_history = res.json::<OpenSeaEventHistory>().await?;

            for listing in event_history.asset_events {
                listings.push(listing);
            }
            Ok(listings)
        }
        StatusCode::GATEWAY_TIMEOUT => {
            warn!("Time out fetching requests, OpenSea is possibly down, retrying in 1s...");
            tokio::time::sleep(Duration::from_secs(1)).await;
            fetch_listings(ctx, after_time).await
        }
        StatusCode::TOO_MANY_REQUESTS => {
            warn!("Rate limited, retrying in 1s...");
            tokio::time::sleep(Duration::from_secs(1)).await;
            fetch_listings(ctx, after_time).await
        }
        _ => {
            warn!("Unexpected response code: {}", res.status());
            Ok(listings)
        }
    }
}

#[async_recursion]
async fn fetch_orders<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    address: String,
    token_id: String,
) -> Result<Vec<opensea::Order>, shared::Error> {
    let res = ctx.handle_os_request(
            ctx
                .http()
                .get(
                    format!(
                        "https://api.opensea.io/wyvern/v1/orders?asset_contract_address={}&token_id={}&payment_token_address=0x{}&taker=0x{}&is_english=false&bundled=false&include_bundled=false&include_invalid=false&side=1&limit=50&offset=0&order_by=eth_price&order_direction=desc",
                        address,
                        token_id,
                        hex::encode(NULL_ADDR.as_bytes()),
                        hex::encode(NULL_ADDR.as_bytes()),
                    )
                )
        ).await?;

    match res.status() {
        StatusCode::OK => {
            let resp = res.text().await?;
            let order_data = match serde_json::from_str::<Orders>(&resp) {
                Ok(order_data) => order_data,
                Err(_) => {
                    dbg!(resp);
                    return Ok(Vec::new());
                }
            };
            // let order_data = res.json::<Order>().await?;
            Ok(order_data.orders)
        }
        StatusCode::GATEWAY_TIMEOUT => {
            warn!("Time out fetching requests, OpenSea is possibly down, retrying in 1s...");
            tokio::time::sleep(Duration::from_secs(1)).await;
            fetch_orders(ctx, address, token_id).await
        }
        StatusCode::TOO_MANY_REQUESTS => {
            warn!("Rate limited, retrying in 1s...");
            tokio::time::sleep(Duration::from_secs(1)).await;
            fetch_orders(ctx, address, token_id).await
        }
        _ => {
            warn!("Unexpected status code: {}", res.status());
            Ok(Vec::new())
        }
    }
}
