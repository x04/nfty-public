use crate::{
    flashbots::BundleRequest,
    rarible::model::{
        activity::{Activity, ActivityType},
        transaction::PreparedTransaction,
    },
    Context,
};
use ethers::{
    abi::Address,
    prelude::{transaction::eip2718::TypedTransaction, *},
};
use log::*;
use shared::{config::SmartGasType, util};
use std::{collections::HashSet, ops::Div, str::FromStr};

pub async fn handle<M: 'static + Middleware + Clone, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    our_addr: Address,
) -> Result<(), shared::Error> {
    let mut is_first_fetch = true;
    let mut checked = HashSet::new();

    loop {
        // by token:
        /*
        let ownerships = ctx
            .http()
            .get(format!(
                "https://api-mainnet.rarible.com/marketplace/api/v4/items/{}:{}/ownerships",
                ctx.config().target.contract_address,
                ctx.config().target.token_id
            ))
            .send()
            .await?
            .json::<Ownerships>()
            .await?;
        */

        // by collection:
        let resp = ctx
            .http()
            .post("https://api-mainnet.rarible.com/marketplace/api/v4/activity")
            .json(&serde_json::json!({
                "filter": {
                    "@type": "by_collection",
                    "address": ctx.config().target.contract_address,
                },
                "size": 100,
                "types": ["ORDER", "CANCEL", "MINT"]
            }))
            .send()
            .await;
        let activity = match resp {
            Ok(resp) => match resp.json::<Activity>().await {
                Ok(acts) => acts,
                Err(e) => {
                    error!("rarible resp dead: {}", e);
                    continue;
                }
            },
            Err(e) => {
                error!("rarible dead: {}", e);
                continue;
            }
        };

        for act in activity
            .into_iter()
            .filter(|a| a.activity_type == ActivityType::Order)
        {
            if checked.contains(&act.id) {
                continue;
            }
            checked.insert(act.id.clone());

            if is_first_fetch {
                continue;
            }

            // TODO: filter out old listings somehow? maybe just ignore first run and only check new?
            if let Some(buy_value) = act.buy_value {
                let base_price = (buy_value / act.value as f64) as i128;
                let minimum_price = (ctx.config().target.minimum_price * 1e18) as i128;
                let maximum_price = (ctx.config().target.maximum_price * 1e18) as i128;
                if base_price >= minimum_price && base_price <= maximum_price
                // value appears to be quantity?
                {
                    dbg!(&act);

                    if let Some(hash) = act.hash {
                        let prepared_tx = ctx
                            .http()
                            .post(format!(
                                "https://api-mainnet.rarible.com/marketplace/api/v4/orders/{}/prepareTransaction",
                                hash
                            ))
                            .json(&serde_json::json!({
                                "amount": act.value.min(ctx.config().target.max_orders as i64),
                                "maker": format!("0x{:x}", our_addr),
                            }))
                            .send()
                            .await?
                            .json::<PreparedTransaction>()
                            .await?;
                        dbg!(&prepared_tx);

                        let (max_fee_per_gas, _max_priority_fee_per_gas) =
                            ctx.provider().estimate_eip1559_fees(None).await?;
                        let adjusted_gas = U256::from(
                            (max_fee_per_gas.as_u64() as f64
                                * ctx.config().transaction.gas_multiplier)
                                as u128,
                        );

                        info!("adjusted gas price: {}", &adjusted_gas.div(1e9 as u64));
                        info!("found matching order");
                        for _ in 0..ctx.config().dev.max_retries {
                            let nonce =
                                ctx.provider().get_transaction_count(our_addr, None).await?;

                            let gas_fee = match ctx.config().target.smart_gas {
                                SmartGasType::Enabled => adjusted_gas.max(U256::from(
                                    (maximum_price - base_price)
                                        / ctx.config().transaction.base_gas_amount as i128,
                                )),
                                SmartGasType::Disabled => adjusted_gas,
                                SmartGasType::Exclusive => U256::from(
                                    (maximum_price - base_price)
                                        / ctx.config().transaction.base_gas_amount as i128,
                                ),
                            };

                            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                                from: Some(our_addr),
                                to: Some(Address::from_str(&prepared_tx.transaction.to)?.into()),
                                value: Some(U256::from_dec_str(&prepared_tx.asset.value)?),
                                data: Some(util::decode_hex(&prepared_tx.transaction.data)?.into()),
                                nonce: Some(nonce),
                                max_priority_fee_per_gas: Some(gas_fee),
                                max_fee_per_gas: Some(gas_fee),
                                ..Default::default()
                            });

                            let gas_amount = if ctx.config().transaction.estimate_gas {
                                match ctx.provider().estimate_gas(&tx).await {
                                    Ok(amount) => U256::from(
                                        (amount.as_u64() as f64
                                            * ctx.config().transaction.gas_amount_multiplier)
                                            as u64,
                                    ),
                                    Err(e) => {
                                        warn!("problem estimating gas amount: {}", e);
                                        U256::from(ctx.config().transaction.base_gas_amount)
                                    }
                                }
                            } else {
                                U256::from(ctx.config().transaction.base_gas_amount)
                            };
                            tx.set_gas(gas_amount);
                            let signature = ctx.provider().signer().sign_transaction(&tx).await?;

                            let mut bundle = BundleRequest::new();
                            bundle.push_transaction(
                                tx.rlp_signed(ctx.provider().signer().chain_id(), &signature),
                            );

                            let block_number = ctx.provider().get_block_number().await?;
                            let target_block = block_number + 1;

                            bundle
                                .set_block(target_block)
                                .set_simulation_block(block_number)
                                .set_simulation_timestamp(util::epoch_time().as_secs());

                            if ctx.config().dev.simulate {
                                match ctx.provider().inner().simulate_bundle(&bundle).await {
                                    Ok(simulated_bundle) => {
                                        dbg!(
                                            target_block,
                                            simulated_bundle.effective_gas_price().as_u64() as f64
                                                / 1e9
                                        );
                                    }
                                    Err(e) => {
                                        error!("error simulating bundle: {}", e);
                                        break;
                                    }
                                }
                            }

                            if ctx.config().dev.dry_run {
                                info!("Dry run, exiting early. Did not send bundle.");
                                break;
                            }

                            if ctx.send_bundle(&bundle).await.is_ok() {
                                break;
                            }
                        }
                    }
                }
            }
        }

        is_first_fetch = false;
        println!("checked");
    }
}
