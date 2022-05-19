use clap::Parser;
use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use k256::{ecdsa::SigningKey, SecretKey};
use std::{convert::TryFrom, path::Path, str::FromStr};

#[derive(Parser)]
#[clap(version = "1.0", author = "cc <md5.eth>")]
struct Opts {
    #[clap(short, long, default_value = "https://eth.721.gg/")]
    provider: String,
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(
        version = "1.0",
        author = "cc <md5.eth>",
        about = "Generate private keys to use for wallets managed by ethernet"
    )]
    Gen(Gen),
    #[clap(
        version = "1.0",
        author = "cc <md5.eth>",
        about = "Send ethereum to wallets"
    )]
    Send(Send),
    #[clap(
        version = "1.0",
        author = "cc <md5.eth>",
        about = "Retrieve ethereum from all wallets managed by ethernet to master wallet"
    )]
    Flush(Flush),
    #[clap(
        version = "1.0",
        author = "cc <md5.eth>",
        about = "Generate configuration files for every private key from a base config file"
    )]
    Configure(Configure),
}

#[derive(Parser)]
struct Gen {
    #[clap(short, long, default_value = "wallets.json")]
    wallets: String,

    #[clap(short, long)]
    count: u64,
}

#[derive(Parser)]
struct Send {
    #[clap(short, long, default_value = "wallets.json")]
    wallets: String,

    #[clap(short, long)]
    from: String,

    #[clap(short, long)]
    amount: f64,
}

#[derive(Parser)]
struct Flush {
    #[clap(short, long, default_value = "wallets.json")]
    wallets: String,

    #[clap(short, long)]
    to: String,
}

#[derive(Parser)]
struct Configure {
    #[clap(short, long, default_value = "wallets.json")]
    wallets: String,

    #[clap(short, long, default_value = "wallets")]
    group: String,

    #[clap(short, long, default_value = "config.toml")]
    base: String,

    #[clap(short, long, default_value = "config-{group}-{index}.toml")]
    format: String,
}

fn generate_key(_: u64) -> (String, String) {
    let key = SecretKey::random(&mut rand::thread_rng());
    let key_bytes = key.to_bytes();

    let signer = SigningKey::from_bytes(&*key_bytes)
        .expect("private key should always be convertible to signing key");
    let address = ethers::utils::secret_key_to_address(&signer);

    (format!("0x{:x}", address), hex::encode(key_bytes))
}

#[tokio::main]
async fn main() -> Result<(), shared::Error> {
    let opts: Opts = Opts::parse();

    let provider = Provider::<Http>::try_from(opts.provider.as_str())?;

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    match opts.verbose {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        _ => println!("Don't be ridiculous"),
    }

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    match opts.subcmd {
        SubCommand::Gen(g) => {
            assert!(g.count > 0);

            if Path::new(&g.wallets).exists() {
                println!("THIS WILL OVERWRITE ANY WALLETS YOU HAVE ALREADY GENERATED!");
                println!("MAKE SURE YOU HAVE CLEARED THEM OR YOU WILL LOSE THOSE FUNDS FOREVER!");
                println!("IF YOU UNDERSTAND THIS PRESS ENTER TO CONTINUE");

                std::io::stdin()
                    .read_line(&mut String::new())
                    .expect("Failed to read line");
            }

            let keys = (0..g.count)
                .into_iter()
                .map(generate_key)
                .collect::<Vec<_>>();
            tokio::fs::write(g.wallets, serde_json::to_string(&keys)?).await?;
        }
        SubCommand::Send(s) => {
            let sender = Wallet::from_str(&s.from)?;
            let client = SignerMiddleware::new(provider, sender);

            let accounts_json = tokio::fs::read(s.wallets).await?;
            let wallets = serde_json::from_slice::<Vec<(String, String)>>(&accounts_json)?;

            let (max_fee_per_gas, max_priority_fee_per_gas) =
                client.estimate_eip1559_fees(None).await?;

            let nonce = client
                .get_transaction_count(client.signer().address(), None)
                .await?;

            let mut pending_txs = Vec::new();
            for (i, w) in wallets.into_iter().enumerate() {
                let to_addr = Address::from_str(&w.0)?;

                let tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                    to: Some(to_addr.into()),
                    gas: Some(U256::from(21000)),
                    nonce: Some(nonce + i),
                    value: Some(U256::from((s.amount * 1e18) as u128)),
                    max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
                    max_fee_per_gas: Some(max_fee_per_gas),
                    ..Default::default()
                });
                pending_txs.push(client.send_transaction(tx, None).await?);
            }
            println!("waiting for txs...");
            futures::future::join_all(pending_txs).await;
            println!("all txs processed!");
        }
        SubCommand::Flush(f) => {
            let to_addr = Address::from_str(&f.to)?;
            let accounts_json = tokio::fs::read(f.wallets).await?;
            let wallets = serde_json::from_slice::<Vec<(String, String)>>(&accounts_json)?;

            let (max_fee_per_gas, max_priority_fee_per_gas) =
                provider.estimate_eip1559_fees(None).await?;

            for w in wallets {
                let sender = Wallet::from_str(&w.1)?;
                let client = SignerMiddleware::new(provider.clone(), sender);
                let balance = client.get_balance(client.signer().address(), None).await?;
                if balance < (max_fee_per_gas * 21001) {
                    continue;
                }
                let tx_value = balance - (max_fee_per_gas * 21001);

                let tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
                    to: Some(to_addr.into()),
                    gas: Some(U256::from(21000)),
                    value: Some(tx_value),
                    max_priority_fee_per_gas: Some(max_priority_fee_per_gas),
                    max_fee_per_gas: Some(max_fee_per_gas),
                    ..Default::default()
                });
                client.send_transaction(tx, None).await?;
            }
            println!("all txs pending!");
        }
        SubCommand::Configure(_) => unimplemented!(),
    }

    Ok(())
}
