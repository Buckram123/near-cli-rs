use dialoguer::{theme::ColorfulTheme, Input, Select};
use interactive_clap::{ToCli, ToInteractiveClapContextScope};
use interactive_clap_derive::{InteractiveClap, ToCliArgs};
use near_primitives::borsh::BorshSerialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod sign_manually;
pub mod sign_with_keychain;
pub mod sign_with_ledger;
pub mod sign_with_private_key;

#[derive(Debug, Clone, EnumDiscriminants, InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SenderContext)]
///Would you like to sign the transaction?
pub enum SignTransaction {
    // /// Provide arguments to sign a private key transaction
    // #[strum_discriminants(strum(
    //     message = "Yes, I want to sign the transaction with my private key"
    // ))]
    // SignPrivateKey(self::sign_with_private_key::SignPrivateKey),
    /// Provide arguments to sign a keychain transaction
    #[strum_discriminants(strum(message = "Yes, I want to sign the transaction with keychain"))]
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    // /// Connect your Ledger device and sign transaction with it
    // #[strum_discriminants(strum(
    //     message = "Yes, I want to sign the transaction with Ledger device"
    // ))]
    // SignWithLedger(self::sign_with_ledger::SignLedger),
    // /// Provide arguments to sign a manually transaction
    // #[strum_discriminants(strum(
    //     message = "No, I want to construct the transaction and sign it somewhere else"
    // ))]
    // SignManually(self::sign_manually::SignManually),
}

impl SignTransaction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            // SignTransaction::SignPrivateKey(keys) => {
            //     keys.process(prepopulated_unsigned_transaction, network_connection_config)
            //         .await
            // }
            SignTransaction::SignWithKeychain(chain) => {
                chain
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            } // SignTransaction::SignWithLedger(ledger) => {
              //     ledger
              //         .process(prepopulated_unsigned_transaction, network_connection_config)
              //         .await
              // }
              // SignTransaction::SignManually(args_manually) => {
              //     args_manually
              //         .process(prepopulated_unsigned_transaction, network_connection_config)
              //         .await
              // }
        }
    }
}

fn input_signer_public_key() -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
    Ok(Input::new()
        .with_prompt("To create an unsigned transaction enter sender's public key")
        .interact_text()
        .unwrap())
}

fn input_signer_private_key() -> color_eyre::eyre::Result<near_crypto::SecretKey> {
    Ok(Input::new()
        .with_prompt("Enter sender's private key")
        .interact_text()
        .unwrap())
}

fn input_access_key_nonce(public_key: &str) -> color_eyre::eyre::Result<u64> {
    println!("Your public key: `{}`", public_key);
    Ok(Input::new()
        .with_prompt(
            "Enter transaction nonce for this public key (query the access key information with \
            `./near-cli view nonce \
                network testnet \
                account 'volodymyr.testnet' \
                public-key ed25519:...` incremented by 1)",
        )
        .interact_text()
        .unwrap())
}

fn input_block_hash() -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
    let input_block_hash: crate::common::BlockHashAsBase58 = Input::new()
        .with_prompt(
            "Enter recent block hash (query information about the hash of the last block with \
            `./near-cli view recent-block-hash network testnet`)",
        )
        .interact_text()
        .unwrap();
    Ok(crate::types::crypto_hash::CryptoHash(
        input_block_hash.inner,
    ))
}

#[derive(Debug, EnumDiscriminants, Clone, clap::Clap, ToCliArgs)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Submit {
    #[strum_discriminants(strum(
        message = "Do you want send the transaction to the server (it's works only for online mode)"
    ))]
    Send,
    #[strum_discriminants(strum(message = "Do you want show the transaction on display?"))]
    Display,
}

impl Submit {
    pub fn choose_submit(connection_config: Option<crate::common::ConnectionConfig>) -> Self {
        println!();
        let variants = SubmitDiscriminants::iter().collect::<Vec<_>>();

        let submits = if let Some(_) = connection_config {
            variants
                .iter()
                .map(|p| p.get_message().unwrap().to_owned())
                .collect::<Vec<_>>()
        } else {
            vec!["Do you want show the transaction on display?".to_string()]
        };
        let select_submit = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action that you want to add to the action:")
            .items(&submits)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_submit] {
            SubmitDiscriminants::Send => Submit::Send,
            SubmitDiscriminants::Display => Submit::Display,
        }
    }

    pub fn process_offline(
        self,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        println!("Serialize_to_base64:\n{}", &serialize_to_base64);
        Ok(None)
    }

    pub async fn process_online(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            Submit::Send => {
                println!("Transaction sent ...");
                let json_rcp_client =
                    near_jsonrpc_client::new_client(network_connection_config.rpc_url().as_str());
                let transaction_info = loop {
                    let transaction_info_result = json_rcp_client
                        .broadcast_tx_commit(near_primitives::serialize::to_base64(
                            signed_transaction
                                .try_to_vec()
                                .expect("Transaction is not expected to fail on serialization"),
                        ))
                        .await;
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(err) => {
                            if let Some(serde_json::Value::String(data)) = &err.data {
                                if data.contains("Timeout") {
                                    println!("Timeout error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
                                    continue;
                                } else {
                                    println!("Error transaction: {:#?}", err)
                                }
                            };
                            return Ok(None);
                        }
                    };
                };
                Ok(Some(transaction_info))
            }
            Submit::Display => {
                println!("\nSerialize_to_base64:\n{}", &serialize_to_base64);
                Ok(None)
            }
        }
    }
}
