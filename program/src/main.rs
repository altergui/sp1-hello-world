//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolType;
use fibonacci_lib::{fibonacci, PublicValuesStruct};
use monotree::database::*;
use monotree::hasher::*;
// use monotree::utils::*;
use monotree::*;

pub fn main() {
    // Read an input to the program.
    //
    // Behind the scenes, this compiles down to a custom system call which handles reading inputs
    // from the prover.
    let n = sp1_zkvm::io::read::<u32>();

    let offset = sp1_zkvm::io::read::<u32>();

    // Compute the n'th fibonacci number using a function from the workspace lib crate.
    let (a, b) = fibonacci(n);

    // offset for fun
    let (a, b) = (a + offset, b + offset);

    let current_id = b.to_string(); // Get current fibonacci number as a String

    // Combine all process IDs
    let mut process_ids = vec![current_id];

    // Init a monotree instance:
    // manually select a db and a hasher as your preference
    // Monotree::<DATABASE, HASHER>::new(DB_PATH)
    // where DATABASE = {MemoryDB, RocksDB, Sled}
    //         HASHER = {Blake3, Blake2s, Blake2b, Sha2, Sha3}
    let mut tree = Monotree::<MemoryDB, Blake3>::new("/tmp/monotree");

    // It is natural the tree root initially has 'None'
    let mut root = None;

    // Prepare a random pair of key and leaf.
    // random_hash() gives a fixed length of random array,
    // where Hash -> [u8; HASH_LEN], HASH_LEN = 32
    // let key = random_hash();
    // let leaf = random_hash();

    // let bu8 = b.try_into().unwrap();
    let key: [u8; 32] = [1; 32];
    let leaf: [u8; 32] = [b as u8; 32];

    for _i in 0..offset {
        // Insert the entry (key, leaf) into tree, yielding a new root of tree
        root = tree
            .insert(root.as_ref(), &key, &leaf)
            .expect("coulnd't insert");
        assert_ne!(root, None);
    }

    // Get the leaf inserted just before. Note that the last root was used.
    let found = tree.get(root.as_ref(), &key).unwrap();
    assert_eq!(found, Some(leaf));

    let root = root.unwrap();
    println!("root: {}", hex::encode(root));

    // Encode the public values of the program.
    let bytes = PublicValuesStruct::abi_encode(&PublicValuesStruct { n, a, b, root });

    // Commit to the public values of the program. The final proof will have a commitment to all the
    // bytes that were committed to.
    sp1_zkvm::io::commit_slice(&bytes);
}
