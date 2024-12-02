use core::fmt;
use std::{collections::HashMap, iter::once};

use eyre::Result;
use itertools::Itertools;
use reth_primitives::{revm_primitives::AccountInfo, Address, Block, Header, B256, U256};
use reth_trie::TrieAccount;
use revm::handler::post_execution::ClearHandle;
use revm_primitives::{keccak256, Bytecode};
use rsp_mpt::{EthereumState, EthereumStateTries, RawEthereumState};
use rsp_witness_db::WitnessDb;
use serde::{
    de::{self, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

/// The input for the client to execute a block and fully verify the STF (state transition
/// function).
///
/// Instead of passing in the entire state, we only pass in the state roots along with merkle proofs
/// for the storage slots that were modified and accessed.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClientExecutorInput {
    /// The current block (which will be executed inside the client).
    pub current_block: Block,
    /// The previous block headers starting from the most recent. There must be at least one header
    /// to provide the parent state root.
    pub ancestor_headers: Vec<Header>,
    /// Network state as of the parent block.
    pub parent_state: EthereumStateTries,
    /// Requests to account state and storage slots.
    pub state_requests: HashMap<Address, Vec<U256>>,
    /// Account bytecodes.
    pub bytecodes: Vec<Bytecode>,
}

pub struct ClientExecutorInputWithRawState {
    pub input_without_state: ClientExecutorInputWithoutParentState,
    pub raw_state: RawEthereumState,
}

impl ClientExecutorInputWithRawState {
    #[inline(always)]
    pub fn parent_header(&self) -> &Header {
        &self.input_without_state.ancestor_headers[0]
    }

    /// Creates a [`WitnessDb`].
    pub fn witness_db(&self) -> Result<WitnessDb> {
        <Self as WitnessInput<RawEthereumState>>::witness_db(self)
    }
}

/// Input for the clint without the parent state.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClientExecutorInputWithoutParentState {
    /// The current block (which will be executed inside the client).
    pub current_block: Block,
    /// The previous block headers starting from the most recent. There must be at least one header
    /// to provide the parent state root.
    pub ancestor_headers: Vec<Header>,
    /// Requests to account state and storage slots.
    pub state_requests: HashMap<Address, Vec<U256>>,
    /// Account bytecodes.
    pub bytecodes: Vec<Bytecode>,
}

impl<'de> Deserialize<'de> for ClientExecutorInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            CurrentBlock,
            AncestorHeaders,
            ParentState,
            StateRequests,
            ByteCodes,
        }

        // This part could also be generated independently by:
        //
        //    #[derive(Deserialize)]
        //    #[serde(field_identifier, rename_all = "lowercase")]
        //    enum Field { Secs, Nanos }
        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("`secs` or `nanos`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "current_block" => Ok(Field::CurrentBlock),
                            "ancestor_headers" => Ok(Field::AncestorHeaders),
                            "parent_state" => Ok(Field::ParentState),
                            "state_requests" => Ok(Field::StateRequests),
                            "bytecodes" => Ok(Field::ByteCodes),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ClientExecutorInputVisitor;

        impl<'de> Visitor<'de> for ClientExecutorInputVisitor {
            type Value = ClientExecutorInput;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct Duration")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<ClientExecutorInput, V::Error>
            where
                V: SeqAccess<'de>,
            {
                println!("cycle-tracker-start: current_block");
                let current_block =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                println!("cycle-tracker-end: current_block");
                println!("cycle-tracker-start: ancestor_headers");
                let ancestor_headers =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                println!("cycle-tracker-end: ancestor_headers");
                println!("cycle-tracker-start: parent_state");
                let parent_state =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                // println!("parent state = {:?}", parent_state);
                println!("cycle-tracker-end: parent_state");
                println!("cycle-tracker-start: state_requests");
                let state_requests =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                println!("cycle-tracker-end: state_requests");
                println!("cycle-tracker-start: bytecodes");
                let bytecodes =
                    seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                println!("cycle-tracker-end: bytecodes");
                Ok(ClientExecutorInput {
                    current_block,
                    ancestor_headers,
                    parent_state,
                    state_requests,
                    bytecodes,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<ClientExecutorInput, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut current_block = None;
                let mut ancestor_headers = None;
                let mut parent_state = None;
                let mut state_requests = None;
                let mut bytecodes = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::CurrentBlock => {
                            if current_block.is_some() {
                                return Err(de::Error::duplicate_field("secs"));
                            }
                            println!("cycle-tracker-start: current_block");
                            current_block = Some(map.next_value()?);
                            println!("cycle-tracker-end: current_block");
                        }
                        Field::AncestorHeaders => {
                            if ancestor_headers.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            println!("cycle-tracker-start: ancestor_headers");
                            ancestor_headers = Some(map.next_value()?);
                            println!("cycle-tracker-end: ancestor_headers");
                        }
                        Field::ParentState => {
                            if parent_state.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            println!("cycle-tracker-start: parent_state");
                            parent_state = Some(map.next_value()?);
                            println!("cycle-tracker-end: parent_state");
                        }
                        Field::StateRequests => {
                            if state_requests.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            println!("cycle-tracker-start: state_requests");
                            state_requests = Some(map.next_value()?);
                            println!("cycle-tracker-end: state_requests");
                        }
                        Field::ByteCodes => {
                            if bytecodes.is_some() {
                                return Err(de::Error::duplicate_field("nanos"));
                            }
                            println!("cycle-tracker-start: bytecodes");
                            bytecodes = Some(map.next_value()?);
                            println!("cycle-tracker-end: bytecodes");
                        }
                    }
                }
                let current_block =
                    current_block.ok_or_else(|| de::Error::missing_field("secs"))?;
                let ancestor_headers =
                    ancestor_headers.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let parent_state = parent_state.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let state_requests =
                    state_requests.ok_or_else(|| de::Error::missing_field("nanos"))?;
                let bytecodes = bytecodes.ok_or_else(|| de::Error::missing_field("nanos"))?;
                Ok(ClientExecutorInput {
                    current_block,
                    ancestor_headers,
                    parent_state,
                    state_requests,
                    bytecodes,
                })
            }
        }

        const FIELDS: &[&str] =
            &["current_block", "ancestor_headers", "parent_state", "state_requests", "bytecodes"];
        deserializer.deserialize_struct("ClientExecutorInput", FIELDS, ClientExecutorInputVisitor)
    }
}

impl ClientExecutorInput {
    /// Gets the immediate parent block's header.
    #[inline(always)]
    pub fn parent_header(&self) -> &Header {
        &self.ancestor_headers[0]
    }

    /// Creates a [`WitnessDb`].
    pub fn witness_db(&self) -> Result<WitnessDb> {
        <Self as WitnessInput<EthereumStateTries>>::witness_db(self)
    }

    pub fn split_parent_state(self) -> (ClientExecutorInputWithoutParentState, EthereumStateTries) {
        (
            ClientExecutorInputWithoutParentState {
                current_block: self.current_block,
                ancestor_headers: self.ancestor_headers,
                state_requests: self.state_requests,
                bytecodes: self.bytecodes,
            },
            self.parent_state,
        )
    }

    pub fn from_split_parent_state(
        input: ClientExecutorInputWithoutParentState,
        parent_state: EthereumStateTries,
    ) -> Self {
        Self {
            current_block: input.current_block,
            ancestor_headers: input.ancestor_headers,
            parent_state,
            state_requests: input.state_requests,
            bytecodes: input.bytecodes,
        }
    }
}

impl WitnessInput<EthereumStateTries> for ClientExecutorInput {
    #[inline(always)]
    fn state(&self) -> &EthereumStateTries {
        &self.parent_state
    }

    #[inline(always)]
    fn state_anchor(&self) -> B256 {
        self.parent_header().state_root
    }

    #[inline(always)]
    fn state_requests(&self) -> impl Iterator<Item = (&Address, &Vec<U256>)> {
        self.state_requests.iter()
    }

    #[inline(always)]
    fn bytecodes(&self) -> impl Iterator<Item = &Bytecode> {
        self.bytecodes.iter()
    }

    #[inline(always)]
    fn headers(&self) -> impl Iterator<Item = &Header> {
        once(&self.current_block.header).chain(self.ancestor_headers.iter())
    }
}

impl WitnessInput<RawEthereumState> for ClientExecutorInputWithRawState {
    #[inline(always)]
    fn state(&self) -> &RawEthereumState {
        &self.raw_state
    }

    #[inline(always)]
    fn state_anchor(&self) -> B256 {
        self.parent_header().state_root
    }

    #[inline(always)]
    fn state_requests(&self) -> impl Iterator<Item = (&Address, &Vec<U256>)> {
        self.input_without_state.state_requests.iter()
    }

    #[inline(always)]
    fn bytecodes(&self) -> impl Iterator<Item = &Bytecode> {
        self.input_without_state.bytecodes.iter()
    }

    #[inline(always)]
    fn headers(&self) -> impl Iterator<Item = &Header> {
        once(&self.input_without_state.current_block.header)
            .chain(self.input_without_state.ancestor_headers.iter())
    }
}

/// A trait for constructing [`WitnessDb`].
pub trait WitnessInput<S: EthereumState> {
    /// Gets a reference to the state from which account info and storage slots are loaded.
    fn state(&self) -> &S;

    /// Gets the state trie root hash that the state referenced by
    /// [state()](trait.WitnessInput#tymethod.state) must conform to.
    fn state_anchor(&self) -> B256;

    /// Gets an iterator over address state requests. For each request, the account info and storage
    /// slots are loaded from the relevant tries in the state returned by
    /// [state()](trait.WitnessInput#tymethod.state).
    fn state_requests(&self) -> impl Iterator<Item = (&Address, &Vec<U256>)>;

    /// Gets an iterator over account bytecodes.
    fn bytecodes(&self) -> impl Iterator<Item = &Bytecode>;

    /// Gets an iterator over references to a consecutive, reverse-chronological block headers
    /// starting from the current block header.
    fn headers(&self) -> impl Iterator<Item = &Header>;

    /// Creates a [`WitnessDb`] from a [`WitnessInput`] implementation. To do so, it verifies the
    /// state root, ancestor headers and account bytecodes, and constructs the account and
    /// storage values by reading against state tries.
    ///
    /// NOTE: For some unknown reasons, calling this trait method directly from outside of the type
    /// implementing this trait causes a zkVM run to cost over 5M cycles more. To avoid this, define
    /// a method inside the type that calls this trait method instead.
    #[inline(always)]
    fn witness_db(&self) -> Result<WitnessDb> {
        println!("cycle-tracker-start: self.state()");
        let state = self.state();
        println!("cycle-tracker-end: self.state()");

        println!("cycle-tracker-start: state_anchor");
        let state_anchor = self.state_anchor();
        println!("cycle-tracker-end: state_anchor");
        println!("cycle-tracker-start: state_root");
        let state_root = state.state_root();
        println!("cycle-tracker-end: state_root");
        println!("cycle-tracker-start: branch1");
        if state_anchor != state_root {
            eyre::bail!("parent state root mismatch");
        }
        println!("cycle-tracker-end: branch1");

        println!("cycle-tracker-start: let bytecodes_by_hash");
        let bytecodes_by_hash =
            self.bytecodes().map(|code| (code.hash_slow(), code)).collect::<HashMap<_, _>>();
        println!("cycle-tracker-end: let bytecodes_by_hash");

        // 0x3af83CaF5fF15D4552cc49Fb20C031E351a009d7 0x3af83CaF5fF15D4552cc49Fb20C031E351a009d7

        let mut accounts = HashMap::new();
        let mut storage = HashMap::new();
        println!("cycle-tracker-start: inserting accounts");
        for (&address, slots) in self.state_requests() {
            let hashed_address = keccak256(address);
            let hashed_address = hashed_address.as_slice();

            let account_in_trie: Option<TrieAccount> = state.get_rlp(hashed_address)?;

            accounts.insert(
                address,
                match account_in_trie {
                    Some(account_in_trie) => AccountInfo {
                        balance: account_in_trie.balance,
                        nonce: account_in_trie.nonce,
                        code_hash: account_in_trie.code_hash,
                        code: Some(
                            (*bytecodes_by_hash
                                .get(&account_in_trie.code_hash)
                                .ok_or_else(|| eyre::eyre!("missing bytecode"))?)
                            // Cloning here is fine as `Bytes` is cheap to clone.
                            .to_owned(),
                        ),
                    },
                    None => Default::default(),
                },
            );

            if !slots.is_empty() {
                let mut address_storage = HashMap::new();

                for &slot in slots {
                    let slot_value = state
                        .get_slot(
                            hashed_address,
                            keccak256(slot.to_be_bytes::<32>().as_slice()).as_slice(),
                        )?
                        .unwrap_or_default();
                    address_storage.insert(slot, slot_value);
                }

                storage.insert(address, address_storage);
            }
        }
        println!("cycle-tracker-end: inserting accounts");

        // Verify and build block hashes
        println!("cycle-tracker-start: Verify and build block hashes");
        let mut block_hashes: HashMap<u64, B256> = HashMap::new();
        for (child_header, parent_header) in self.headers().tuple_windows() {
            if parent_header.number != child_header.number - 1 {
                eyre::bail!("non-consecutive blocks");
            }

            if parent_header.hash_slow() != child_header.parent_hash {
                eyre::bail!("parent hash mismatch");
            }

            block_hashes.insert(parent_header.number, child_header.parent_hash);
        }
        println!("cycle-tracker-end: Verify and build block hashes");

        Ok(WitnessDb { accounts, storage, block_hashes })
    }
}
