#![allow(dead_code, unused_imports)]

use crate::{flashbots::BundleRequest, Context, Error};
use autosolve::types::CaptchaTokenRequest;
use chrono::{Duration, NaiveDateTime, Utc};
use deno_core::{error::AnyError, Extension, FsModuleLoader, OpState};
use deno_runtime::{
    deno_broadcast_channel::InMemoryBroadcastChannel,
    deno_web::BlobStore,
    permissions::{Permissions, PermissionsOptions},
    worker::{MainWorker, WorkerOptions},
    BootstrapOptions,
};
use ethers::prelude::{transaction::eip2718::TypedTransaction, *};
use ethers_core::abi::ParamType;
use itertools::Itertools;
use log::*;
use serde::Deserialize;
use shared::{
    config::{IncludeAddressType, Mint, MintArgument, MintMode},
    contracts,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    str::FromStr,
    sync::{Arc, Mutex as StdMutex},
};
use tokio::{
    sync::{oneshot, Mutex},
    task::LocalSet,
};

#[derive(Debug, Deserialize)]
struct MintInfo {
    function: Option<String>,
    arguments: Option<Vec<MintArgument>>,
    raw: Option<String>,
}

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

async fn op_send_autosolve<M: 'static + Middleware + Clone, S: 'static + Signer + Clone>(
    state: Rc<RefCell<OpState>>,
    token_request: CaptchaTokenRequest,
    _: (),
) -> Result<String, AnyError> {
    let autosolve_client = {
        let state = state.borrow_mut();
        state.try_borrow::<autosolve::Client>().unwrap().clone()
    };

    let token_res = autosolve_client
        .send_token_request(token_request)
        .await
        .map_err(|_| AnyError::msg("error sending request"))?
        .await
        .map_err(|_| AnyError::msg("error receiving response"))?;
    Ok(token_res.token)
}

fn op_new_autosolve_request(
    _: &mut OpState,
    _: (),
    _: (),
) -> Result<CaptchaTokenRequest, AnyError> {
    Ok(CaptchaTokenRequest::default())
}

fn op_get_address(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    Ok(format!(
        "{}",
        ethers::utils::to_checksum(state.try_borrow::<Address>().unwrap(), None)
    ))
}

fn op_get_config(state: &mut OpState, _: (), _: ()) -> Result<Mint, AnyError> {
    Ok(state.try_borrow::<Mint>().unwrap().clone())
}

fn op_return_data(state: &mut OpState, value: MintInfo, _: ()) -> Result<(), AnyError> {
    let tx = state
        .try_borrow::<Arc<StdMutex<Option<oneshot::Sender<MintInfo>>>>>()
        .unwrap();
    tx.lock().unwrap().take().unwrap().send(value).unwrap();
    Ok(())
}

pub async fn handle<M: 'static + Middleware + Clone, S: 'static + Signer + Clone>(
    ctx: Context<M, S>,
    our_addr: Address,
) -> Result<(), shared::Error> {
    let mint_config = ctx.config().mint.as_ref().expect("expected Mint config");
    let included_txs = Arc::new(Mutex::new(HashMap::new()));

    let pool_monitor_active = if mint_config.mode == MintMode::Flashbots {
        let pool_ctx = ctx.clone();
        let pool_mint_config = mint_config.clone();
        let pool_included_txs = included_txs.clone();

        let include_type = pool_mint_config
            .include_address_type
            .clone()
            .unwrap_or(IncludeAddressType::From);
        let include_address = pool_mint_config
            .include_address
            .as_ref()
            .and_then(|a| Address::from_str(a).ok());
        let include_method = if let Some(sig) = pool_mint_config.include_method {
            if sig.is_empty() {
                None
            } else if sig.starts_with("0x") {
                Some(shared::util::decode_hex(&sig)?)
            } else {
                Some(shared::contracts::function_identifier(sig).to_vec())
            }
        } else {
            None
        };

        if include_address.is_none() && include_method.is_none() {
            false
        } else {
            tokio::spawn(async move {
                info!("starting tx pool monitor");
                loop {
                    info!("fetching tx pool");
                    let mut matching_txs = HashMap::new();
                    let txpool = pool_ctx.provider().txpool_content().await.unwrap();
                    let pending_txs = txpool
                        .pending
                        .values()
                        .map(|x| x.values().collect_vec())
                        .concat();
                    let queued_txs = txpool
                        .queued
                        .values()
                        .map(|x| x.values().collect_vec())
                        .concat();
                    for tx in queued_txs.into_iter().chain(pending_txs.into_iter()) {
                        if let Some(include_address) = include_address {
                            match include_type {
                                IncludeAddressType::From => {
                                    if tx.from != include_address {
                                        continue;
                                    }
                                }
                                IncludeAddressType::To => {
                                    if tx.to != Some(include_address) {
                                        continue;
                                    }
                                }
                            }
                        }

                        if let Some(include_method) = include_method.clone() {
                            if !tx.input.0.starts_with(&include_method) {
                                continue;
                            }
                        }

                        info!("found matching include tx: 0x{:x}", tx.hash);
                        matching_txs.insert(tx.hash, tx.rlp());
                    }

                    *pool_included_txs.lock().await = matching_txs;
                    info!("updated tx pool");
                }
            });

            true
        }
    } else {
        false
    };

    if mint_config.mode == MintMode::Normal {
        if let Some(start_time) = mint_config.start_time {
            if start_time > 0 {
                info!("sleeping until 1s before start time...");
                let until_drop =
                    NaiveDateTime::from_timestamp(start_time as i64, 0) - Utc::now().naive_utc();
                tokio::time::sleep(until_drop.to_std()?).await;
            }
        }

        if let Some(state_checks) = mint_config.state_checks.as_ref() {
            if !state_checks.is_empty() {
                let mut sale_started = true;
                for check in state_checks {
                    let resp = ctx
                        .provider()
                        .call(
                            &TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                to: Some(NameOrAddress::Address(Address::from_str(
                                    check
                                        .address
                                        .as_ref()
                                        .unwrap_or(&mint_config.contract_address),
                                )?)),
                                data: Some(
                                    contracts::encode_call(
                                        &check.function,
                                        check
                                            .arguments
                                            .iter()
                                            .map(|x| x.r#type.to_token(&x.value).unwrap())
                                            .collect::<Vec<_>>()
                                            .as_slice(),
                                    )
                                    .into(),
                                ),
                                ..Default::default()
                            }),
                            None,
                        )
                        .await?;

                    let expected_response = contracts::encode_args(
                        check
                            .return_value
                            .iter()
                            .map(|x| x.r#type.to_token(&x.value).unwrap())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    )
                    .to_vec();

                    if resp.to_vec() != expected_response {
                        sale_started = false;
                    }
                }

                if sale_started {
                    info!("sale has started, sending txs...");
                } else {
                    info!("waiting for sale start...");
                    let mut block_sub = ctx.provider().watch_blocks().await.unwrap();
                    'state_check: loop {
                        info!("waiting for next block...");
                        let block = block_sub.next().await;
                        if block.is_none() {
                            return Err("error watching blocks".into());
                        }

                        for check in state_checks {
                            let resp = ctx
                                .provider()
                                .call(
                                    &TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                        to: Some(NameOrAddress::Address(Address::from_str(
                                            check
                                                .address
                                                .as_ref()
                                                .unwrap_or(&mint_config.contract_address),
                                        )?)),
                                        data: Some(
                                            contracts::encode_call(
                                                &check.function,
                                                check
                                                    .arguments
                                                    .iter()
                                                    .map(|x| x.r#type.to_token(&x.value).unwrap())
                                                    .collect::<Vec<_>>()
                                                    .as_slice(),
                                            )
                                            .into(),
                                        ),
                                        ..Default::default()
                                    }),
                                    None,
                                )
                                .await?;

                            let expected_response = contracts::encode_args(
                                check
                                    .return_value
                                    .iter()
                                    .map(|x| x.r#type.to_token(&x.value).unwrap())
                                    .collect::<Vec<_>>()
                                    .as_slice(),
                            )
                            .to_vec();

                            if resp.to_vec() != expected_response {
                                warn!("sale is not live");
                                continue 'state_check;
                            }
                        }

                        break 'state_check;
                    }
                }
            }
        }
    }

    let calldata = generate_calldata(ctx.clone(), mint_config).await;

    loop {
        match mint_config.mode {
            MintMode::Flashbots => loop {
                let nonce = ctx.provider().get_transaction_count(our_addr, None).await?;

                let mut bundles = Vec::new();

                if pool_monitor_active {
                    let included_txs = { included_txs.lock().await.clone() };
                    if !included_txs.is_empty() {
                        for raw_tx in included_txs.values().cloned() {
                            let mut bundle = BundleRequest::new();
                            bundle.push_transaction(raw_tx);
                            bundles.push(bundle);
                        }
                    } else {
                        bundles.push(BundleRequest::new());
                    }
                } else {
                    bundles.push(BundleRequest::new());
                }

                let gas_fee = U256::from((mint_config.gas_fee * 1e9) as u128);
                let total_txs = mint_config.transaction_count.unwrap_or(1);

                let block_number = ctx.provider().get_block_number().await?;
                let target_block = block_number + 1;

                let value = if let Some(price_function) = mint_config.price_function.as_ref() {
                    let resp = ctx
                        .provider()
                        .call(
                            &TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                to: Some(NameOrAddress::Address(Address::from_str(
                                    price_function
                                        .address
                                        .as_ref()
                                        .unwrap_or(&mint_config.contract_address),
                                )?)),
                                data: Some(
                                    contracts::encode_call(
                                        &price_function.function,
                                        price_function
                                            .arguments
                                            .iter()
                                            .map(|x| x.r#type.to_token(&x.value).unwrap())
                                            .collect::<Vec<_>>()
                                            .as_slice(),
                                    )
                                    .into(),
                                ),
                                ..Default::default()
                            }),
                            None,
                        )
                        .await?;

                    let resp =
                        ethers::abi::decode(&[ParamType::Uint(256)], resp.to_vec().as_slice())?;
                    if let Some(ethers::abi::Token::Uint(a)) = resp.get(0) {
                        U256::from(a.as_u128()) * U256::from(price_function.multiplier)
                    } else {
                        unreachable!();
                    }
                } else {
                    U256::from(mint_config.value as u128)
                };

                for bundle in bundles.iter_mut() {
                    for i in 0..total_txs {
                        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                            from: Some(our_addr),
                            to: Some(Address::from_str(&mint_config.contract_address)?.into()),
                            value: Some(value),
                            data: Some(calldata.clone().to_vec().into()),
                            nonce: Some(nonce + i),
                            max_priority_fee_per_gas: Some(
                                mint_config
                                    .priority_fee
                                    .map(|x| U256::from((x * 1e9) as u128))
                                    .unwrap_or(gas_fee),
                            ),
                            max_fee_per_gas: Some(gas_fee),
                            gas: None,
                            ..Default::default()
                        });

                        let gas_limit = match mint_config.gas_limit {
                            Some(limit) => U256::from(limit),
                            None => panic!("gas estimation not available for flashbots"),
                        };

                        tx.set_gas(gas_limit);

                        let signature = ctx.provider().signer().sign_transaction(&tx).await?;
                        bundle.push_transaction(
                            tx.rlp_signed(ctx.provider().signer().chain_id(), &signature),
                        );
                    }

                    bundle
                        .set_block(target_block)
                        .set_simulation_block(block_number)
                        .set_simulation_timestamp(shared::util::epoch_time().as_secs());

                    if let Some(start_time) = mint_config.start_time {
                        if start_time > 0 {
                            bundle
                                .set_min_timestamp(start_time + 1)
                                .set_max_timestamp(
                                    start_time + Duration::hours(8).num_seconds() as u64,
                                )
                                .set_simulation_timestamp(start_time + 1);
                        }
                    }

                    if ctx.config().account.simulate {
                        let simulated_bundle = ctx.provider().inner().simulate_bundle(bundle).await;
                        match simulated_bundle {
                            Ok(bundle) => {
                                dbg!(&bundle);
                                dbg!(
                                    target_block,
                                    bundle.effective_gas_price().as_u128() as f64 / 1e9,
                                );
                            }
                            Err(e) => {
                                warn!("error simulating bundle: {}", e);
                            }
                        }
                    }

                    if ctx.config().account.dry_run {
                        info!("Dry run, exiting early. Did not send bundle.");
                        break;
                    }

                    if ctx.send_bundle(bundle).await.is_ok() {
                        break;
                    }
                }
            },
            MintMode::Normal => {
                let nonce = match mint_config.initial_nonce {
                    Some(nonce) if nonce > 0 => U256::from(nonce),
                    None | Some(_) => ctx.provider().get_transaction_count(our_addr, None).await?,
                };

                dbg!(nonce);

                let mut transactions = Vec::new();

                if mint_config.bump_mempool.unwrap_or(false) {
                    if let Ok(mempool) = ctx.provider().txpool_content().await {
                        let gas_fee = U256::from((mint_config.gas_fee * 1e9) as u128);
                        if let Some(pending_txs) = mempool
                            .pending
                            .get(&our_addr)
                            .map(|x| x.values().collect_vec())
                        {
                            for tx in pending_txs {
                                let tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                    from: Some(our_addr),
                                    to: Some(tx.to.unwrap().into()),
                                    value: Some(tx.value),
                                    data: Some(tx.input.clone()),
                                    nonce: Some(tx.nonce),
                                    max_priority_fee_per_gas: Some(
                                        mint_config
                                            .priority_fee
                                            .map(|x| U256::from((x * 1e9) as u128))
                                            .unwrap_or(gas_fee),
                                    ),
                                    max_fee_per_gas: Some(gas_fee),
                                    gas: Some(tx.gas),
                                    ..Default::default()
                                });

                                let signature =
                                    ctx.provider().signer().sign_transaction(&tx).await?;
                                transactions.push(hex::encode(
                                    &tx.rlp_signed(ctx.provider().signer().chain_id(), &signature)
                                        .to_vec(),
                                ));
                            }
                        }

                        if let Some(queued_txs) = mempool
                            .queued
                            .get(&our_addr)
                            .map(|x| x.values().collect_vec())
                        {
                            for tx in queued_txs {
                                let tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                    from: Some(our_addr),
                                    to: Some(tx.to.unwrap().into()),
                                    value: Some(tx.value),
                                    data: Some(tx.input.clone()),
                                    nonce: Some(tx.nonce),
                                    max_priority_fee_per_gas: Some(
                                        mint_config
                                            .priority_fee
                                            .map(|x| U256::from((x * 1e9) as u128))
                                            .unwrap_or(gas_fee),
                                    ),
                                    max_fee_per_gas: Some(gas_fee),
                                    gas: Some(tx.gas),
                                    ..Default::default()
                                });

                                let signature =
                                    ctx.provider().signer().sign_transaction(&tx).await?;
                                transactions.push(hex::encode(
                                    &tx.rlp_signed(ctx.provider().signer().chain_id(), &signature)
                                        .to_vec(),
                                ));
                            }
                        }
                    }
                } else {
                    let value = if let Some(price_function) = mint_config.price_function.as_ref() {
                        let resp = ctx
                            .provider()
                            .call(
                                &TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                    to: Some(NameOrAddress::Address(Address::from_str(
                                        price_function
                                            .address
                                            .as_ref()
                                            .unwrap_or(&mint_config.contract_address),
                                    )?)),
                                    data: Some(
                                        contracts::encode_call(
                                            &price_function.function,
                                            price_function
                                                .arguments
                                                .iter()
                                                .map(|x| x.r#type.to_token(&x.value).unwrap())
                                                .collect::<Vec<_>>()
                                                .as_slice(),
                                        )
                                        .into(),
                                    ),
                                    ..Default::default()
                                }),
                                None,
                            )
                            .await?;

                        let resp =
                            ethers::abi::decode(&[ParamType::Uint(256)], resp.to_vec().as_slice())?;
                        if let Some(ethers::abi::Token::Uint(a)) = resp.get(0) {
                            U256::from(a.as_u128()) * U256::from(price_function.multiplier)
                        } else {
                            unreachable!();
                        }
                    } else {
                        U256::from(mint_config.value as u128)
                    };

                    let mut failed_simulation = false;
                    let gas_fee = U256::from((mint_config.gas_fee * 1e9) as u128);
                    for i in 0..mint_config.transaction_count.unwrap_or(1).max(1) {
                        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                            from: Some(our_addr),
                            to: Some(Address::from_str(&mint_config.contract_address)?.into()),
                            value: Some(value),
                            data: Some(calldata.clone().to_vec().into()),
                            nonce: Some(nonce + i),
                            max_priority_fee_per_gas: Some(
                                mint_config
                                    .priority_fee
                                    .map(|x| U256::from((x * 1e9) as u128))
                                    .unwrap_or(gas_fee),
                            ),
                            max_fee_per_gas: Some(gas_fee),
                            gas: None,
                            ..Default::default()
                        });

                        let gas_limit = match mint_config.gas_limit {
                            Some(limit) => U256::from(limit),
                            None => match ctx.provider().estimate_gas(&tx).await {
                                Ok(gas_used) => {
                                    U256::from((gas_used.as_u64() as f64 * 1.1).round() as u64)
                                }
                                Err(_) => {
                                    error!("failed to estimate gas limit, waiting for next block to try again...");
                                    failed_simulation = true;
                                    break;
                                }
                            },
                        };

                        tx.set_gas(gas_limit);

                        if ctx.config().account.simulate {
                            if let Err(e) = ctx.provider().call(&tx, None).await {
                                error!("transaction simulation failed, waiting for next block to try again. ({})", e);
                                failed_simulation = true;
                                break;
                            }
                        }

                        let signature = ctx.provider().signer().sign_transaction(&tx).await?;
                        transactions.push(hex::encode(
                            &tx.rlp_signed(ctx.provider().signer().chain_id(), &signature)
                                .to_vec(),
                        ));
                    }

                    if failed_simulation {
                        'simulate: loop {
                            let mut block_subscription = ctx.provider().watch_blocks().await?;
                            'block: while let Some(_) = block_subscription.next().await {
                                transactions.clear();
                                for i in 0..mint_config.transaction_count.unwrap_or(1).max(1) {
                                    let mut tx =
                                        TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                            from: Some(our_addr),
                                            to: Some(
                                                Address::from_str(&mint_config.contract_address)?
                                                    .into(),
                                            ),
                                            value: Some(value),
                                            data: Some(calldata.clone().to_vec().into()),
                                            nonce: Some(nonce + i),
                                            max_priority_fee_per_gas: Some(
                                                mint_config
                                                    .priority_fee
                                                    .map(|x| U256::from((x * 1e9) as u128))
                                                    .unwrap_or(gas_fee),
                                            ),
                                            max_fee_per_gas: Some(gas_fee),
                                            gas: None,
                                            ..Default::default()
                                        });

                                    let gas_limit = match mint_config.gas_limit {
                                        Some(limit) => U256::from(limit),
                                        None => match ctx.provider().estimate_gas(&tx).await {
                                            Ok(gas_used) => U256::from(
                                                (gas_used.as_u64() as f64 * 1.1).round() as u64,
                                            ),
                                            Err(_) => {
                                                error!("failed to estimate gas limit, waiting for next block to try again...");
                                                break;
                                            }
                                        },
                                    };

                                    tx.set_gas(gas_limit);

                                    if ctx.config().account.simulate {
                                        if let Err(e) = ctx.provider().call(&tx, None).await {
                                            error!("transaction simulation failed, waiting for next block to try again. ({})", e);
                                            continue 'block;
                                        }
                                    }

                                    let signature =
                                        ctx.provider().signer().sign_transaction(&tx).await?;
                                    transactions.push(hex::encode(
                                        &tx.rlp_signed(
                                            ctx.provider().signer().chain_id(),
                                            &signature,
                                        )
                                        .to_vec(),
                                    ));
                                }
                                break 'simulate;
                            }
                        }
                    }
                }

                dbg!(transactions.len());

                if ctx.config().account.dry_run {
                    info!("Dry run, exiting early. Did not send transaction(s).");
                    return Ok(());
                }

                let metamask_provider = Provider::<Http>::try_from(
                    "https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161",
                )?;

                for tx in transactions {
                    loop {
                        let raw_tx = hex::decode(&tx).unwrap();
                        let mm = metamask_provider.clone();
                        let mm_tx = raw_tx.clone();
                        tokio::spawn(async move {
                            let _ = mm.send_raw_transaction(mm_tx.into()).await;
                        });
                        let temp_ctx = ctx.clone();
                        tokio::spawn(async move {
                            let _ = temp_ctx
                                .provider()
                                .send_raw_transaction(raw_tx.into())
                                .await;
                        });
                        let resp = ctx
                            .http()
                            .post("https://www.google.com/transaction")
                            .header(
                                "authorization",
                                format!(
                                    "{}-{}",
                                    ctx.credentials().token_id.unwrap_or(0),
                                    ctx.session()
                                ),
                            )
                            .json(&serde_json::json!({
                                "transaction": tx,
                            }))
                            .send()
                            .await
                            .and_then(|r| r.error_for_status());
                        match resp {
                            Ok(resp) => {
                                info!("tx submitted: {}", resp.text().await?);
                                break;
                            }
                            Err(e) => {
                                error!("error submitting tx: {}", e);
                                info!("resubmitting tx in 1s...");
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                    }
                }

                info!("all transactions submitted successfully, exiting.");

                return Ok(());
            }
        }
    }
}

async fn generate_calldata<M: 'static + Middleware + Clone, S: 'static + Signer + Clone>(
    ctx: Context<M, S>,
    mint_config: &Mint,
) -> Vec<u8> {
    match mint_config.script_identifier.as_ref() {
        Some(script_identifier) if !script_identifier.is_empty() => {
            let script_identifier = script_identifier.clone();
            let (tx, rx) = oneshot::channel::<MintInfo>();
            let tx = Arc::new(StdMutex::new(Some(tx)));

            let state_address = ctx.provider().signer().address();
            let state_autosolve = ctx.autosolve().cloned();
            let state_config = mint_config.clone();

            let rt = tokio::runtime::Handle::current();
            std::thread::spawn(move || {
                rt.block_on(async {
                    let module_loader = Rc::new(FsModuleLoader);
                    let create_web_worker_cb = Arc::new(|_| {
                        todo!("Web workers are not supported in the example");
                    });

                    let options = WorkerOptions {
                        bootstrap: BootstrapOptions {
                            apply_source_maps: false,
                            args: vec![],
                            cpu_count: 1,
                            debug_flag: false,
                            enable_testing_features: false,
                            location: None,
                            no_color: false,
                            runtime_version: "x".into(),
                            ts_version: "x".into(),
                            unstable: false,
                        },
                        extensions: vec![Extension::builder()
                            .state(move |state| {
                                state.put(state_address);
                                if let Some(autosolve) = state_autosolve.as_ref() {
                                    state.put(autosolve.clone());
                                }
                                state.put(state_config.clone());
                                state.put(tx.clone());
                                Ok(())
                            })
                            .build()],
                        unsafely_ignore_certificate_errors: None,
                        root_cert_store: None,
                        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.81 Safari/537.36".into(),
                        seed: None,
                        js_error_create_fn: None,
                        create_web_worker_cb,
                        maybe_inspector_server: None,
                        should_break_on_first_statement: false,
                        module_loader,
                        get_error_class_fn: Some(&get_error_class_name),
                        origin_storage_dir: None,
                        blob_store: BlobStore::default(),
                        broadcast_channel: InMemoryBroadcastChannel::default(),
                        shared_array_buffer_store: None,
                        compiled_wasm_module_store: None,
                    };

                    let script_url = format!("https://static.721.gg/{}.js", &script_identifier);
                    let main_module = deno_core::resolve_url(&script_url).unwrap();
                    let permissions = Permissions::from_options(&PermissionsOptions {
                        allow_hrtime: true,
                        allow_env: Some(vec![]),
                        allow_net: Some(vec![]),
                        allow_ffi: None,
                        allow_read: None,
                        allow_run: None,
                        allow_write: None,
                        prompt: false,
                    });

                    let mut worker = MainWorker::bootstrap_from_options(main_module, permissions, options);
                    worker.js_runtime.register_op(
                        "sendAutosolve",
                        deno_core::op_async(op_send_autosolve::<M, S>),
                    );
                    worker.js_runtime.register_op(
                        "newAutosolveRequest",
                        deno_core::op_sync(op_new_autosolve_request),
                    );
                    worker
                        .js_runtime
                        .register_op("getAddress", deno_core::op_sync(op_get_address));
                    worker
                        .js_runtime
                        .register_op("getConfig", deno_core::op_sync(op_get_config));
                    worker
                        .js_runtime
                        .register_op("returnTxData", deno_core::op_sync(op_return_data));
                    worker.js_runtime.sync_ops_cache();

                    let local = LocalSet::new();
                    local.spawn_local(async move {
                        worker
                            .execute_script(
                                &format!("{}.js", &script_identifier),
                                &reqwest::get(&script_url)
                                    .await
                                    .expect("could not fetch information")
                                    .error_for_status()
                                    .expect("could not fetch information")
                                    .text()
                                    .await
                                    .expect("could not fetch information"),
                            )
                            .unwrap();
                        worker.run_event_loop(false).await.unwrap();
                    });
                    local.await;
                });
            });

            let mint_info = rx.await.unwrap();

            let calldata = match (mint_info.function, mint_info.arguments, mint_info.raw) {
                (Some(function), Some(arguments), None) => shared::contracts::encode_call(
                    &function,
                    arguments
                        .iter()
                        .map(|x| x.r#type.to_token(&x.value).unwrap())
                        .collect::<Vec<_>>()
                        .as_slice(),
                ),
                (None, None, Some(raw)) => shared::util::decode_hex(&raw).unwrap(),
                _ => unreachable!(),
            };
            dbg!(hex::encode(&calldata));
            calldata
        }
        _ => shared::contracts::encode_call(
            &mint_config.function,
            mint_config
                .arguments
                .iter()
                .map(|x| x.r#type.to_token(&x.value).unwrap())
                .collect::<Vec<_>>()
                .as_slice(),
        ),
    }
}
