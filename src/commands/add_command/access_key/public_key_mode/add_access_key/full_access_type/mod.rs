/// данные для определения ключа с полным доступом
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliFullAccessType {
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
// #[interactive_clap(context = super::super::super::sender::SenderContext)]
pub struct FullAccessType {
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CliFullAccessType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.sign_option
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<FullAccessType> for CliFullAccessType {
    fn from(full_access_type: FullAccessType) -> Self {
        Self {
            sign_option: Some(full_access_type.sign_option.into()),
        }
    }
}

// --- Временная имплементация
impl From<super::super::super::sender::SenderContext> for crate::common::Context {
    fn from(item: super::super::super::sender::SenderContext) -> Self {
        Self {
            connection_config: item.connection_config,
            sender_account_id: Some(item.sender_account_id),
        }
    }
}
// ---------------------------

impl FullAccessType {
    pub fn from(
        item: CliFullAccessType,
        context: super::super::super::sender::SenderContext,
        // connection_config: Option<crate::common::ConnectionConfig>,
        // sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        // let sender_account_id = context.sender_account_id;
        let common_context = crate::common::Context::from(context); // Временно
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(Some(cli_sign_transaction), common_context)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_variant(common_context)?,
        };
        Ok(Self { sign_option })
    }
}

impl FullAccessType {
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: public_key.clone(),
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        println!(
            "\nAdding full access key = {:?} to {:?}.",
            public_key, unsigned_transaction.signer_id
        );
        match self
            .sign_option
            .process(
                unsigned_transaction.clone(),
                network_connection_config.clone(),
            )
            .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                )
                .await;
            }
            None => {}
        };
        Ok(())
    }
}
