use dialoguer::Input;
use interactive_clap::{ToCli, ToInteractiveClapContextScope};
use interactive_clap_derive::InteractiveClap;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

#[derive(Debug, Clone, EnumDiscriminants, InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
#[interactive_clap(disable_strum_discriminants)]
pub enum Transfer {
    /// Enter an amount to transfer
    Amount(TransferNEARTokensAction),
}

impl Transfer {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            Transfer::Amount(transfer_near_action) => {
                transfer_near_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, InteractiveClap)]
#[interactive_clap(context = super::sender::SenderContext)]
pub struct TransferNEARTokensAction {
    pub amount: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl ToCli for crate::common::NearBalance {
    type CliVariant = crate::common::NearBalance;
}

impl TransferNEARTokensAction {
    fn from(
        optional_clap_variant: Option<<TransferNEARTokensAction as ToCli>::CliVariant>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let amount: crate::common::NearBalance = match context.connection_config.clone() {
            Some(network_connection_config) => {
                let account_balance: crate::common::NearBalance =
                    match crate::common::check_account_id(
                        network_connection_config.clone(),
                        context.clone().sender_account_id.into(),
                    )? {
                        Some(account_view) => {
                            crate::common::NearBalance::from_yoctonear(account_view.amount)
                        }
                        None => crate::common::NearBalance::from_yoctonear(0),
                    };
                match optional_clap_variant
                    .clone()
                    .and_then(|clap_variant| clap_variant.amount)
                {
                    Some(cli_amount) => {
                        if cli_amount <= account_balance {
                            cli_amount
                        } else {
                            println!(
                                "You need to enter a value of no more than {}",
                                account_balance
                            );
                            TransferNEARTokensAction::input_amount(Some(account_balance))
                        }
                    }
                    None => TransferNEARTokensAction::input_amount(Some(account_balance)),
                }
            }
            None => match optional_clap_variant
                .clone()
                .and_then(|clap_variant| clap_variant.amount)
            {
                Some(cli_amount) => cli_amount,
                None => TransferNEARTokensAction::input_amount(None),
            },
        };
        let sign_option = match optional_clap_variant.and_then(|clap_variant| clap_variant.sign_option) {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(Some(cli_sign_transaction), context)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_variant(context)?,
        };
        Ok(Self {
            amount,
            sign_option,
        })
    }
}

impl TransferNEARTokensAction {
    fn input_amount(
        account_balance: Option<crate::common::NearBalance>,
    ) -> crate::common::NearBalance {
        match account_balance {
            Some(account_balance) => loop {
                let input_amount: crate::common::NearBalance = Input::new()
                            .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                            .with_initial_text(format!("{}", account_balance))
                            .interact_text()
                            .unwrap();
                if input_amount <= account_balance {
                    break input_amount;
                } else {
                    println!(
                        "You need to enter a value of no more than {}",
                        account_balance
                    )
                }
            }
            None => Input::new()
                        .with_prompt("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)")
                        .interact_text()
                        .unwrap()
        }
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: self.amount.to_yoctonear(),
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config.clone())
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
