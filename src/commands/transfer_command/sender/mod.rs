use crate::common::SenderContext;
use dialoguer::Input;
use interactive_clap::{ToCli, ToInteractiveClapContextScope};
use interactive_clap_derive::InteractiveClap;

#[derive(Debug, Clone, InteractiveClap)]
#[interactive_clap(input_context = super::operation_mode::TransferCommandNetworkContext)]
#[interactive_clap(output_context = crate::common::SenderContext)]
pub struct Sender {
    pub sender_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    pub send_to: super::receiver::Receiver,
}

impl crate::common::SenderContext {
    pub fn from_previous_context(
        previous_context: super::operation_mode::TransferCommandNetworkContext,
        scope: &<Sender as ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            connection_config: previous_context.connection_config.clone(),
            sender_account_id: scope.sender_account_id.clone(),
        }
    }
}

impl Sender {
    fn input_sender_account_id(
        context: &super::operation_mode::TransferCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let connection_config = context.connection_config.clone();
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the account ID of the sender?")
                .interact_text()
                .unwrap();
            if let Some(connection_config) = &connection_config {
                if let Some(_) = crate::common::check_account_id(
                    connection_config.clone(),
                    account_id.clone().into(),
                )? {
                    break Ok(account_id);
                } else {
                    println!("Account <{}> doesn't exist", account_id.to_string());
                }
            } else {
                break Ok(account_id);
            }
        }
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.0.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
