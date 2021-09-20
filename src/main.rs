use anyhow::Result;
use ethers::core::types::transaction::eip2718::TypedTransaction;
use ethers::prelude::*;
use ethers_flashbots::*;
use std::convert::TryFrom;
use url::Url;
use std::env;


#[tokio::main]
async fn main() -> Result<()> {

    // Connect to the network - using URL used by metamask
    let provider = Provider::<Http>::try_from("https://goerli.infura.io/v3/33ff530a5bfc4b418314cd6b5cc6fc64")?;

    let private_key = env::var("TEST_PRIVATE_KEY")?;
    let bundle_signer = private_key
        .parse::<LocalWallet>()?;
    let wallet = private_key
        .parse::<LocalWallet>()?;

    // Set chainId for goerli testnet
    let wallet = wallet.with_chain_id(5u64);
    let bundle_signer = bundle_signer.with_chain_id(5u64);

    let client = SignerMiddleware::new(
        FlashbotsMiddleware::new(
            provider,
            Url::parse("https://relay-goerli.flashbots.net")?,
            bundle_signer,
        ),
        wallet,
    );

    let bundle = get_bundle_for_test(&client).await?;
    let current_block_number = client.inner().inner().get_block_number().await?;
    let bundle= bundle
        .set_simulation_block(current_block_number)
        .set_simulation_timestamp(1731851886)
        .set_block(current_block_number+1);
    let simulated_bundle = client.inner().simulate_bundle(&bundle).await?;
    println!("Simulated bundle: {:?}", simulated_bundle);

    // submitting multiple bundles to increase the probability on inclusion
    for x in 0..10 {
        let bundle = get_bundle_for_test(&client).await?;
        let bundle = bundle
            .set_block(current_block_number + x);
        println!("Bundle Initialized");
        println!("{}",current_block_number + x);
        let pending_bundle = client.inner().send_bundle(&bundle).await?;
        match pending_bundle.await {
            Ok(bundle_hash) => println!(
                "Bundle with hash {:?} was included in target block",
                bundle_hash
            ),
            Err(PendingBundleError::BundleNotIncluded) => {
                println!("Bundle was not included in target block.")
            }
            Err(e) => println!("An error occured: {}", e),
        }
    }

    Ok(())
}

async fn get_bundle_for_test<M: 'static + Middleware, S: 'static + Signer>(client: &SignerMiddleware<M, S>) -> Result<BundleRequest, >
{
    let mut nounce = client.get_transaction_count(client.address(), None).await?;

    let mut tx: TypedTransaction = TransactionRequest::pay("vitalik.eth", 100).into();
    let bundle = BundleRequest::new();
    // creation bundle with multiple transaction to handle the gas spent in a bundle > 42000
    let bundle = {
        tx.set_nonce(nounce);
        client.fill_transaction(&mut tx, None).await?;
        nounce = nounce + 1;
        let signature = client.signer().sign_transaction(&tx).await?;
        let inner = bundle.push_transaction(tx.rlp_signed(client.signer().chain_id(), &signature));
        inner
    };
    let bundle = {
        tx.set_nonce(nounce);
        client.fill_transaction(&mut tx, None).await?;
        let signature = client.signer().sign_transaction(&tx).await?;
        let inner = bundle.push_transaction(tx.rlp_signed(client.signer().chain_id(), &signature));
        inner
    };
    Ok(bundle)
}
