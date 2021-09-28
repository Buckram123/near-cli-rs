use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::{display_access_key_list, display_account_info, ConnectionConfig};
use near_primitives::types::{AccountId, Finality};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = super::operation_mode::online_mode::select_server::ViewAccountSummaryCommandNetworkContext)]
///Choose Block ID
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this contract at final block"))]
    /// Specify a block ID final to view this contract
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this contract at block heigt"))]
    /// Specify a block ID height to view this contract
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View this contract at block hash"))]
    /// Specify a block ID hash to view this contract
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl BlockId {
    pub async fn process(
        self,
        account_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(account_id, network_connection_config)
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(account_id, network_connection_config)
                    .await
            }
            Self::AtFinalBlock => {
                display_account_info(account_id.clone(), &network_connection_config)
                    .await?;
                display_access_key_list(account_id.clone(), &network_connection_config)
                    .await?;
                Ok(())
            }
        }
    }
}
