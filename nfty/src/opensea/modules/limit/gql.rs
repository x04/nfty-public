use crate::{
    flashbots::BundleRequest,
    model::{EventHistoryNode, OldOrder, OpenSeaEventHistory, Order},
    opensea::{gql, gql::Query},
    util, Context, Error,
};
use chrono::{DateTime, Utc};
use ethers::prelude::*;
use itertools::Itertools;
use log::*;
use rand::{prelude::*, thread_rng};
use shared::config::{OSLimitCollection, OSLimitMode, SmartGas};
use std::collections::HashMap;
use tokio::time::Duration;

pub async fn handle<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
) -> Result<(), Error> {
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
) -> Result<(), Error> {
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
        .expect("expected collections")
        .clone()
        .into_iter()
        .fold(
            HashMap::<String, Vec<OSLimitCollection>>::new(),
            |mut m, c| match m.get(&c.slug) {
                Some(list) => {
                    let mut list = list.clone();
                    list.push(c.clone());
                    m.insert(c.slug, list.clone());
                    m
                }
                None => {
                    m.insert(c.slug.clone(), vec![c]);
                    m
                }
            },
        );

    let executor = gql::Executor::from_config(ctx.config())?;
    let mut last_time = Some(Utc::now());
    loop {
        info!("fetching new listings...");

        let cur_time = Utc::now();
        let listings = match fetch_listings(&executor, &collections, last_time).await {
            Ok(listings) => listings,
            Err(e) => {
                error!("error fetching new listings: {}", e);
                continue;
            }
        };
        last_time = Some(cur_time);

        let mut filtered_listings = listings
            .into_iter()
            .filter(|l| {
                l.ending_price.is_some()
                    && l.price.is_some()
                    && l.asset_quantity.is_some()
                    && l.price.as_ref().unwrap().asset.symbol == *"ETH"
            })
            .collect_vec();

        if filtered_listings.is_empty() {
            ctx.delay("no new listings").await;
            continue;
        }

        filtered_listings.shuffle(&mut thread_rng());

        for l in filtered_listings {
            let asset_quantity = l.asset_quantity.as_ref().unwrap();
            let collection_name = asset_quantity.asset.collection.slug.to_lowercase();

            let listing_price = l.price.unwrap();
            let price = match listing_price.quantity.parse::<i128>() {
                Ok(p) => p,
                Err(e) => {
                    error!(
                        "error parsing price for listing, val: {}, error: {}",
                        listing_price.quantity, e
                    );
                    continue;
                }
            };
            let (matches_rule, maximum_price) = collections
                .get(&collection_name)
                .unwrap()
                .iter()
                .fold((false, U256::default()), |(found, max_price), c| {
                    if found {
                        (found, max_price)
                    } else {
                        if price < (c.minimum_price * 1e18) as i128
                            || price > (c.maximum_price * 1e18) as i128
                        {
                            (false, U256::default())
                        } else {
                            /*
                            let traits = asset_quantity
                                .asset
                                .traits
                                .edges
                                .iter()
                                .map(|e| e.node.clone())
                                .collect_vec();
                            let has_traits = c
                                .traits
                                .as_ref()
                                .map(|t| {
                                    t.is_empty()
                                        || t.iter().all(|trait_filter| {
                                            let matching_trait = traits
                                                .iter()
                                                .find(|tn| tn.trait_type == trait_filter.name);
                                            match trait_filter.r#type {
                                                OSLimitTraitType::Include => match matching_trait {
                                                    Some(tn) => {
                                                        trait_filter.value == *"*"
                                                            || tn
                                                                .value
                                                                .as_ref()
                                                                .map(|v| v.to_lowercase())
                                                                == Some(
                                                                    trait_filter
                                                                        .value
                                                                        .to_lowercase(),
                                                                )
                                                    }
                                                    None => false,
                                                },
                                                OSLimitTraitType::Exclude => match matching_trait {
                                                    Some(tn) => {
                                                        trait_filter.value != *"*"
                                                            && tn
                                                                .value
                                                                .as_ref()
                                                                .map(|v| v.to_lowercase())
                                                                != Some(
                                                                    trait_filter
                                                                        .value
                                                                        .to_lowercase(),
                                                                )
                                                    }
                                                    None => true,
                                                },
                                            }
                                        })
                                })
                                .unwrap_or(true);
                             */
                            (true, U256::from((c.maximum_price * 1e18) as i128))
                        }
                    }
                });
            if !matches_rule {
                warn!("listing does not match any rules, skipping...");
                continue;
            }

            info!(
                "found potential order matching min/max for token id {} in collection {} @ {} eth",
                &asset_quantity.asset.token_id,
                &asset_quantity.asset.collection.slug,
                price as f64 / 1e18
            );

            let orders_resp = fetch_orders(
                &executor,
                asset_quantity.asset.contract.address.clone(),
                asset_quantity.asset.token_id.clone(),
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

            let filtered_orders = orders
                .into_iter()
                .filter(|o| o.taker.address == *"0x0000000000000000000000000000000000000000");

            for order in filtered_orders {
                let base_price = U256::from_dec_str(&order.base_price)?;
                if base_price > maximum_price {
                    continue;
                } else if order.listing_time >= shared::util::epoch_time().as_secs() {
                    warn!("found order that has listing time in the future, probably an auction. ignoring.");
                    continue;
                }

                info!("found matching order");
                send_tx(ctx, our_addr, maximum_price, base_price, &order).await?;
                break;
            }
        }

        ctx.delay("processed new listings").await;
    }
}

async fn token_loop<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
) -> Result<(), Error> {
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

    let executor = gql::Executor::from_config(ctx.config())?;
    loop {
        info!("fetching orders...");

        let orders_resp = fetch_orders(
            &executor,
            limit_config
                .contract_address
                .as_ref()
                .expect("expected contract_address")
                .clone(),
            limit_config
                .token_id
                .as_ref()
                .expect("expected token_id")
                .clone(),
        )
        .await;

        let mut orders = match orders_resp {
            Ok(orders) => orders,
            Err(e) => {
                error!("error fetching orders: {}", e);
                continue;
            }
        };
        if orders.is_empty() {
            ctx.delay_warn("found no orders for asset").await;
            continue;
        }

        orders.shuffle(&mut rand::thread_rng());

        let filtered_orders = orders
            .into_iter()
            .filter(|o| o.taker.address == *"0x0000000000000000000000000000000000000000");

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

        ctx.delay("processed new listings").await;
    }
}

async fn send_tx<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
    maximum_price: U256,
    base_price: U256,
    order: &OldOrder,
) -> Result<(), Error> {
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

        let tx = util::order_to_tx(
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

async fn fetch_listings(
    executor: &gql::Executor,
    collections: &HashMap<String, Vec<OSLimitCollection>>,
    after_time: Option<DateTime<Utc>>,
) -> Result<Vec<EventHistoryNode>, Error> {
    let mut cursor = String::new();
    let mut listings = Vec::new();
    loop {
        let res = executor
            .execute(Query::new(
                "EventHistoryPollQuery",
                gql::EVENT_HISTORY_QUERY,
                serde_json::json!({
                    "archetype": None as Option<String>,
                    "categories": None as Option<String>,
                    "chains": None as Option<String>,
                    "collections": collections.keys().collect::<Vec<_>>(),
                    "eventTimestamp_Gt": after_time.map(|t| t.format("%Y-%m-%dT%T%.6f").to_string()),
                    "count": 100,
                    "cursor": &cursor,
                    "identity": None as Option<String>,
                    "showAll": true,
                }),
            ))
            .await?;

        // for some reason it seems like once we fetch all assets it just says we're rate limited?
        // so this is a sort of hacky "fix" for that, not really sure how else to handle it atm
        match res.status_code {
            // GATEWAY TIMEOUT
            504 => {
                warn!("Time out fetching requests, OpenSea is possibly down, retrying in 1s...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            // TOO MANY REQUESTS
            429 => {
                warn!("Rate limited, retrying in 1s...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                break;
            }
            _ => {}
        }

        let event_history = match serde_json::from_slice::<OpenSeaEventHistory>(&res.body) {
            Ok(event_history) => event_history,
            Err(e) => {
                unsafe {
                    dbg!(e, std::str::from_utf8_unchecked(&res.body));
                }
                return Ok(listings);
            }
        };
        // let event_history = res.json::<OpenSeaEventHistory>().await?;

        for listing in event_history
            .data
            .asset_events
            .edges
            .into_iter()
            .map(|e| e.node)
        {
            listings.push(listing);
        }

        if event_history
            .data
            .asset_events
            .page_info
            .as_ref()
            .map(|page| page.has_next_page)
            .unwrap_or(false)
        {
            cursor = event_history
                .data
                .asset_events
                .page_info
                .unwrap()
                .end_cursor
                .unwrap();
        } else {
            break;
        }
    }

    Ok(listings)
}

async fn fetch_orders(
    executor: &gql::Executor,
    address: String,
    token_id: String,
) -> Result<Vec<OldOrder>, Error> {
    let mut orders = Vec::new();
    loop {
        let res = executor
            .execute(Query::new(
                "OrdersQuery",
                gql::ORDERS_QUERY,
                serde_json::json!({
                    "cursor": null,
                    "count": 10,
                    "excludeMaker": null,
                    "isExpired": false,
                    "isValid": true,
                    "maker": null,
                    "makerAssetIsPayment": null,
                    "takerArchetype": null,
                    "takerAssetCategories": null,
                    "takerAssetCollections": null,
                    "takerAssetIsOwnedBy": null,
                    "takerAssetIsPayment": true,
                    "sortAscending": true,
                    "sortBy": "TAKER_ASSETS_USD_PRICE",
                    "makerAssetBundle": null,
                    "takerAssetBundle": null,
                    "expandedMode": false,
                    "isBid": false,
                    "filterByOrderRules": false,
                    "makerArchetype": {
                        "assetContractAddress": address,
                        "tokenId": token_id,
                        "chain": "ETHEREUM",
                    },
                }),
            ))
            .await?;

        match res.status_code {
            // GATEWAY TIMEOUT
            504 => {
                warn!("Time out fetching requests, OpenSea is possibly down, retrying in 1s...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            // TOO MANY REQUESTS
            429 => {
                warn!("Rate limited, retrying in 1s...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
            _ => {}
        }

        let order_data = match serde_json::from_slice::<Order>(&res.body) {
            Ok(order_data) => order_data,
            Err(_) => {
                unsafe {
                    dbg!(std::str::from_utf8_unchecked(&res.body));
                }
                return Ok(orders);
            }
        };
        // let order_data = res.json::<Order>().await?;

        for edge in order_data.data.orders.edges {
            orders.push(
                match serde_json::from_str::<OldOrder>(&edge.node.old_order) {
                    Ok(order_data) => order_data,
                    Err(e) => {
                        dbg!(e, &edge.node.old_order);
                        return Ok(orders);
                    }
                },
            );
        }
        break;
    }
    Ok(orders)
}
