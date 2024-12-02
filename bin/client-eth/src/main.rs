#![no_main]
sp1_zkvm::entrypoint!(main);

// use epserde::prelude::*;
use rsp_client_executor::{io::{ClientExecutorInput, ClientExecutorInputWithoutParentState}, ClientExecutor, EthereumVariant};
use rsp_mpt::RawEthereumState;

const LA_CONSTANTE: [usize; 12] = [0; 12];

pub fn main() {
    println!("{:?}", LA_CONSTANTE);

    // test epserde
    // let s = [0_usize; 1000];

    // // Serialize it
    // let mut file = std::env::temp_dir();
    // file.push("serialized0");
    // s.serialize(&mut std::fs::File::create(&file)?)?;
    // // Load the serialized form in a buffer
    // let b = std::fs::read(&file)?;

    // // The type of t will be inferred--it is shown here only for clarity
    // let t: &[usize; 1000] = <[usize; 1000]>::deserialize_eps(b.as_ref())?;

    // assert_eq!(s, *t);

    println!("Si, me llamo octopopopo");
    // Read the input.
    println!("cycle-tracker-start: toa la wea");
    println!("cycle-tracker-start: sp1_zkvm::io::read_vec()");
    letprintln!("cycle-tracker-end: sp1_zkvm::io::read_vec()");
    input_without_state = sp1_zkvm::io::read_vec();
    println!("cycle-tracker-end: sp1_zkvm::io::read_vec()");

    println!("cycle-tracker-start: sp1_zkvm::io::read_vec()");
    let raw_state = sp1_zk_vm::io::read_vec();

    println!("cycle-tracker-end: sp1_zkvm::io::read_vec()");

    println!("cycle-tracker-start: searializando");
    let input = bincode::deserialize::<ClientExecutorInputWithoutParentState>(&input_without_state).unwrap();
    println!("cycle-tracker-end: searializando");

    let raw_state = RawEthereumState::from_raw_data(raw_state);

    // Execute the block.
    let executor = ClientExecutor;
    let header = executor.execute::<EthereumVariant>((input, raw_state)).expect("failed to execute client");
    let block_hash = header.hash_slow();

    // Commit the block hash.
    sp1_zkvm::io::commit(&block_hash);

    println!("cycle-tracker-end: toa la wea");
}
