use crate::cache::BlockchainDb;
use crate::error::DatabaseError;
use crate::types::{ToAlloy, ToEthers};
use alloy_primitives::{keccak256, Address, Bytes, B256, U256};
use ethers_core::types::{BigEndianHash, BlockId, NameOrAddress};
use ethers_providers::Middleware;
use revm::{
    db::DatabaseRef,
    primitives::{AccountInfo, Bytecode, KECCAK_EMPTY},
};

pub struct Backend<M: Middleware> {
    pub provider: M,
    pub db: BlockchainDb,
    pub block_id: Option<BlockId>,
}

impl<M: Middleware> DatabaseRef for Backend<M> {
    type Error = DatabaseError;

    fn basic(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let acc = self.db.accounts().read().get(&address).cloned();
        if let Some(basic) = acc {
            Ok(Some(basic))
        } else {
            let add = NameOrAddress::Address(address.to_ethers());

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let balance = rt
                .block_on(self.provider.get_balance(add.clone(), self.block_id))
                .unwrap();
            let nonce = rt
                .block_on(
                    self.provider
                        .get_transaction_count(add.clone(), self.block_id),
                )
                .unwrap();
            let code = rt
                .block_on(self.provider.get_code(add, self.block_id))
                .unwrap();
            let code = Bytes::from(code.0);

            let (code, code_hash) = if !code.is_empty() {
                (code.clone(), keccak256(&code))
            } else {
                (Bytes::default(), KECCAK_EMPTY)
            };

            let account_info = AccountInfo {
                balance: balance.to_alloy(),
                nonce: nonce.as_u64(),
                code_hash,
                code: Some(Bytecode::new_raw(code).to_checked()),
            };

            self.db
                .accounts()
                .write()
                .insert(address, account_info.clone());
            Ok(Some(account_info))
        }
    }

    fn code_by_hash(&self, hash: B256) -> Result<Bytecode, Self::Error> {
        Err(DatabaseError::MissingCode(hash))
    }

    fn storage(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        let value = self
            .db
            .storage()
            .read()
            .get(&address)
            .and_then(|acc| acc.get(&index).copied());
        if let Some(value) = value {
            Ok(value)
        } else {
            let block_id = self.block_id;
            let idx_req = B256::from(index);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let storage = rt
                .block_on(self.provider.get_storage_at(
                    NameOrAddress::Address(address.to_ethers()),
                    idx_req.to_ethers(),
                    block_id,
                ))
                .unwrap();
            let storage = storage.into_uint().to_alloy();
            self.db
                .storage()
                .write()
                .entry(address)
                .or_default()
                .insert(index, storage);
            Ok(storage)
        }
    }

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        let hash = self
            .db
            .block_hashes()
            .read()
            .get(&U256::from(number))
            .cloned();
        if let Some(hash) = hash {
            Ok(hash)
        } else {
            let n: u64 = number.try_into().unwrap();
            let block_id = BlockId::from(n);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            let block = rt.block_on(self.provider.get_block(block_id));

            match block {
                Ok(Some(block)) => Ok(block
                    .hash
                    .expect("empty block hash on mined block, this should never happen")
                    .to_alloy()),
                Ok(None) => {
                    // if no block was returned then the block does not exist, in which case
                    // we return empty hash
                    Ok(KECCAK_EMPTY)
                }
                Err(_) => Err(DatabaseError::BlockNotFound(block_id)),
            }
        }
    }
}
