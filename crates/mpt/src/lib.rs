use eyre::Result;
use reth_trie::{AccountProof, HashedPostState, TrieAccount};
use revm::primitives::{Address, HashMap, B256};
use revm_primitives::U256;
use serde::{Deserialize, Serialize};

/// Module containing MPT code adapted from `zeth`.
mod mpt;
use mpt::{proofs_to_tries, transition_proofs_to_tries, Error, MptNode};

pub trait EthereumState {
    fn state_root(&self) -> B256;

    fn get_rlp<T: alloy_rlp::Decodable>(&self, key: &[u8]) -> Result<Option<T>, Error>;

    fn get_slot(&self, address_key: &[u8], slot_key: &[u8]) -> Result<Option<U256>, Error>;
}

pub trait StorageTrie {}

/// Ethereum state trie and account storage tries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EthereumStateTries {
    pub state_trie: MptNode,
    pub storage_tries: HashMap<B256, MptNode>,
}

impl EthereumStateTries {
    /// Builds Ethereum state tries from relevant proofs before and after a state transition.
    pub fn from_transition_proofs(
        state_root: B256,
        parent_proofs: &HashMap<Address, AccountProof>,
        proofs: &HashMap<Address, AccountProof>,
    ) -> Result<Self> {
        transition_proofs_to_tries(state_root, parent_proofs, proofs)
            .map_err(|err| eyre::eyre!("{}", err))
    }

    /// Builds Ethereum state tries from relevant proofs from a given state.
    pub fn from_proofs(state_root: B256, proofs: &HashMap<Address, AccountProof>) -> Result<Self> {
        proofs_to_tries(state_root, proofs).map_err(|err| eyre::eyre!("{}", err))
    }

    /// Mutates state based on diffs provided in [`HashedPostState`].
    pub fn update(&mut self, post_state: &HashedPostState) {
        for (hashed_address, account) in post_state.accounts.iter() {
            let hashed_address = hashed_address.as_slice();

            match account {
                Some(account) => {
                    let state_storage = &post_state.storages.get(hashed_address).unwrap();
                    let storage_root = {
                        let storage_trie = self.storage_tries.get_mut(hashed_address).unwrap();

                        if state_storage.wiped {
                            storage_trie.clear();
                        }

                        for (key, value) in state_storage.storage.iter() {
                            let key = key.as_slice();
                            if value.is_zero() {
                                storage_trie.delete(key).unwrap();
                            } else {
                                storage_trie.insert_rlp(key, *value).unwrap();
                            }
                        }

                        storage_trie.hash()
                    };

                    let state_account = TrieAccount {
                        nonce: account.nonce,
                        balance: account.balance,
                        storage_root,
                        code_hash: account.get_bytecode_hash(),
                    };
                    self.state_trie.insert_rlp(hashed_address, state_account).unwrap();
                }
                None => {
                    self.state_trie.delete(hashed_address).unwrap();
                }
            }
        }
    }

    pub fn to_bytes(self) -> Vec<u8> {
        match self.state_trie.as_data() {
            mpt::MptNodeData::Null => vec![0u8],
            mpt::MptNodeData::Branch(childrens) => {
                let mut bytes = vec![]
                for node in childrens {
                    if let Some(node) = node
                        bytes.extend((**node).to_bytes());
                    } else {
                        bytes.push(0u8);
                    }
                }
                bytes
            },
            mpt::MptNodeData::Leaf(vec, vec1) => todo!(),
            mpt::MptNodeData::Extension(vec, mpt_node) => todo!(),
            mpt::MptNodeData::Digest(fixed_bytes) => todo!(),
        }
    }

}

impl EthereumState for EthereumStateTries {
    /// Computes the state root.
    fn state_root(&self) -> B256 {
        println!("cycle-tracker-start: keccak");
        let hash = self.state_trie.hash();
        println!("cycle-tracker-end: keccak");
        hash
    }

    fn get_rlp<T: alloy_rlp::Decodable>(&self, key: &[u8]) -> Result<Option<T>, Error> {
        self.state_trie.get_rlp(key)
    }

    fn get_slot(&self, address_key: &[u8], slot_key: &[u8]) -> Result<Option<U256>, Error> {
        let storage_trie = self.storage_tries.get(address_key).unwrap(); // TODO: Handle error
        storage_trie.get_rlp(slot_key)
    }
}

#[derive(Debug)]
pub struct RawEthereumState {
    pub raw_data: Vec<u8>,
}

impl RawEthereumState {
    fn root_from_index(&self, index: usize) -> B256 {
        let node_type = self.raw_data[index];
        unimplemented!()
    }
}

impl EthereumState for RawEthereumState {
    fn state_root(&self) -> B256 {
        self.root_from_index(0)
    }

    fn get_rlp<T: alloy_rlp::Decodable>(&self, key: &[u8]) -> Result<Option<T>, Error> {
        todo!()
    }

    fn get_slot(&self, address_key: &[u8], slot_key: &[u8]) -> Result<Option<U256>, Error> {
        todo!()
    }
}
