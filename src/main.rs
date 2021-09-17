use anyhow::Result;
use ethers::core::types::transaction::eip2718::TypedTransaction;
use ethers::prelude::*;
use ethers_flashbots::*;
use std::convert::TryFrom;
use url::Url;
use std::env;


#[tokio::main]
async fn main() -> Result<()> {
    // Connect to the network
    let provider = Provider::<Http>::try_from("https://goerli.infura.io/v3/33ff530a5bfc4b418314cd6b5cc6fc64")?;
    println!("Provider Initialized");
    // This is your searcher identity
    let priv_key = env::var("TEST_PRIVATE_KEY")?;
    let bundle_signer = priv_key
        .parse::<LocalWallet>()?;
    // This signs transactions
    let wallet = priv_key
        .parse::<LocalWallet>()?;
    println!("Signers Initialized");

    // Add signer and Flashbots middleware
    let client = SignerMiddleware::new(
        FlashbotsMiddleware::new(
            provider,
            Url::parse("https://relay-goerli.flashbots.net")?,
            bundle_signer,
        ),
        wallet,
    );
    println!("Client Initialized");

    let tx = {
        let mut inner: TypedTransaction = TransactionRequest::pay("vitalik.eth", 100).into();
        client.fill_transaction(&mut inner, None).await?;
        inner
    };
    println!("Txn Initialized");

    let signature = client.signer().sign_transaction(&tx).await?;
    let current_block_number = client.inner().inner().get_block_number().await?;
    // let bundle = BundleRequest::new()
    //     .push_transaction(tx.rlp_signed(client.signer().chain_id(), &signature))
    //     .set_block(current_block_number + 1)
    //     .set_simulation_block(current_block_number + 1)
    //     .set_simulation_timestamp(1731851886);
    println!("Bundle Initialized");
    // let simulated_bundle = client.inner().simulate_bundle(&bundle).await?;
    // println!("Bundle Simulated");
    // println!("Simulated bundle: {:?}", simulated_bundle);

    for x in 0..10 {
        let bundle = BundleRequest::new()
            .push_transaction(tx.rlp_signed(client.signer().chain_id(), &signature))
            .set_block(current_block_number + x);
        println!("Bundle Initialized");
        let pending_bundle = client.inner().send_bundle(&bundle).await?;
    }

    Ok(())
}