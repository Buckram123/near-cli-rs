use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::operation_mode::AddAccessKeyCommandNetworkContext)]
#[interactive_clap(output_context = crate::common::SenderContext)]
#[interactive_clap(fn_from_cli = default)]
pub struct Sender {
    pub sender_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Specify public key
    pub public_key: super::DeleteAccessKeyType,
}

impl crate::common::SenderContext {
    pub fn from_previous_context_for_delete_access_key(
        previous_context: super::operation_mode::DeleteAccessKeyCommandNetworkContext,
        scope: &<Sender as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            connection_config: previous_context.connection_config,
            sender_account_id: scope.sender_account_id.clone(),
        }
    }
}

impl Sender {
    pub fn from_cli(
        optional_clap_variant: Option<CliSender>,
        context: super::operation_mode::DeleteAccessKeyCommandNetworkContext,
    ) -> color_eyre::eyre::Result<Self> {
        let sender_account_id: near_primitives::types::AccountId = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.sender_account_id) {
            Some(cli_sender_account_id) => match &context.connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    network_connection_config,
                    cli_sender_account_id.clone(),
                )? {
                    Some(_) => sender_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", sender_account_id);
                        Sender::input_sender_account_id(&context)?
                    }
                },
                None => sender_account_id,
            },
            None => Self::input_sender_account_id(&context)?,
        };
        type Alias = <Sender as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope;
        let new_context_scope = Alias { sender_account_id };
        let new_context = crate::common::SenderContext::from_previous_context_for_delete_access_key(
            context,
            &new_context_scope,
        );
        let public_key = super::DeleteAccessKeyType::from_cli(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.public_key {
                Some(ClapNamedArgDeleteAccessKeyTypeForSender::PublicKey(cli_arg)) => Some(cli_arg),
                None => None,
            }),
            new_context,
        )?;
        Ok(Self {
            sender_account_id: new_context_scope.sender_account_id,
            public_key,
        })
    }
}

impl Sender {
    fn input_sender_account_id(
        context: &super::operation_mode::DeleteAccessKeyCommandNetworkContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("Which account ID do you need to remove the key from?")
                .interact_text()
                .unwrap();
            if let Some(connection_config) = &context.connection_config {
                if crate::common::get_account_state(connection_config, account_id.clone())?.is_some() {
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
            signer_id: self.sender_account_id.clone().into(),
            receiver_id: self.sender_account_id.clone().into(),
            ..prepopulated_unsigned_transaction
        };
        self.public_key
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
