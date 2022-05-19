mod gql;
mod model;

use crate::{
    looksrare::modules::limit::{gql::Executor, model::Event},
    Context, Error,
};
use ethers::prelude::*;
use log::*;
use std::time::Duration;

pub async fn handle<M: 'static + Middleware, S: 'static + Signer + Clone>(
    ctx: &Context<M, S>,
    _our_addr: Address,
) -> Result<(), Error> {
    let executor = gql::Executor::from_config(ctx.config())?;
    loop {
        let events_res = fetch_events(&executor).await;
        match events_res {
            Ok(events) => handle_events(&events).await?,
            Err(e) => error!("error fetching results: {e}"),
        }
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }
}

async fn fetch_events(executor: &Executor) -> Result<Vec<Event>, Error> {
    let resp = executor
        .execute(gql::Query::new(
            "",
            gql::GET_EVENTS_QUERY,
            serde_json::json!({
              "filter": {
                "collection": "0xED5AF388653567Af2F388E6224dC7C4b3241C544",
                "type": ["LIST"]
              },
              "pagination": {
                "first": 100
              }
            }),
        ))
        .await?;
    let events = resp
        .error_for_status()?
        .json::<model::GetEventsResponse>()
        .await?
        .data
        .and_then(|x| x.events)
        .unwrap_or_else(|| Vec::new());
    Ok(events)
}

async fn handle_events(events: &[Event]) -> Result<(), Error> {
    if events.is_empty() {
        return Ok(());
    }

    let id = events.first().and_then(|e| e.id.as_ref()).unwrap();
    info!("{id}");
    Ok(())
}
