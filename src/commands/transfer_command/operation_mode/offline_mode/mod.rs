use interactive_clap::{ToCli, ToInteractiveClapContextScope};
use interactive_clap_derive::InteractiveClap;

#[derive(Debug, Clone, InteractiveClap)]
// #[interactive_clap(input_context = (), output_context = super::TransferCommandNetworkContext)]
#[interactive_clap(input_context = (), output_context = OfflineArgsContext)]
pub struct OfflineArgs {
    #[interactive_clap(named_arg)]
    send_from: super::super::sender::Sender,
}

struct OfflineArgsContext {}

impl OfflineArgsContext {
    fn from_previous_context(
        previous_context: (),
        scope: <OfflineArgs as ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {}
    }
}

impl From<OfflineArgsContext> for super::TransferCommandNetworkContext {
    fn from(item: OfflineArgsContext) -> Self {
        Self {
            connection_config: None,
        }
    }
}

impl OfflineArgs {
    pub fn from(
        optional_clap_variant: Option<CliOfflineArgs>,
        context: (),
    ) -> color_eyre::eyre::Result<Self> {
        let new_context_scope = InteractiveClapContextScopeForOfflineArgs {};
        let new_context = OfflineArgsContext::from_previous_context(context, new_context_scope);
        let send_from = super::super::sender::Sender::from(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.send_from {
                Some(ClapNamedArgSenderForOfflineArgs::SendFrom(cli_sender)) => Some(cli_sender),
                None => None,
            }),
            new_context.into(),
        )?;
        Ok(Self { send_from })
    }
}

impl OfflineArgs {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let selected_server_url = None;
        self.send_from
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
