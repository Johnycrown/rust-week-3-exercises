use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct CompactSize {
    pub value: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BitcoinError {
    InsufficientBytes,
    InvalidFormat,
}

impl CompactSize {
    pub fn new(value: u64) -> Self {
        let mut encoded = Vec::new();
        if value < 0xFD {
            encoded.push(value as u8);
        } else if value <= 0xFFFF {
            encoded.push(0xFD);
            encoded.extend_from_slice(&(value as u16).to_le_bytes());
        } else if value <= 0xFFFF_FFFF {
            encoded.push(0xFE);
            encoded.extend_from_slice(&(value as u32).to_le_bytes());
        } else {
            encoded.push(0xFF);
            encoded.extend_from_slice(&value.to_le_bytes());
        }
        CompactSize { value }
    }
}


    pub fn to_bytes(&self) -> Vec<u8> {
    let value = self.value;
    let mut encoded = Vec::new();

    if value < 0xFD {
        // [0x00–0xFC] => 1 byte
        encoded.push(value as u8);
    } else if value <= 0xFFFF {
        // [0xFDxxxx] => 0xFD + u16 (2 bytes)
        encoded.push(0xFD);
        encoded.extend_from_slice(&(value as u16).to_le_bytes());
    } else if value <= 0xFFFF_FFFF {
        // [0xFExxxxxxxx] => 0xFE + u32 (4 bytes)
        encoded.push(0xFE);
        encoded.extend_from_slice(&(value as u32).to_le_bytes());
    } else {
        // [0xFFxxxxxxxxxxxxxxxx] => 0xFF + u64 (8 bytes)
        encoded.push(0xFF);
        encoded.extend_from_slice(&value.to_le_bytes());

    encoded
}

//     pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
//         // TODO: Decode CompactSize, returning value and number of bytes consumed.
//         // First check if bytes is empty.
//         // Check that enough bytes are available based on prefix.
//     }
// }

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct Txid(pub [u8; 32]);

// impl Serialize for Txid {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         // TODO: Serialize as a hex-encoded string (32 bytes => 64 hex characters)
//     }
// }

// impl<'de> Deserialize<'de> for Txid {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         // TODO: Parse hex string into 32-byte array
//         // Use `hex::decode`, validate length = 32
//     }
// }

// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// pub struct OutPoint {
//     pub txid: Txid,
//     pub vout: u32,
// }

// impl OutPoint {
//     pub fn new(txid: [u8; 32], vout: u32) -> Self {
//         // TODO: Create an OutPoint from raw txid bytes and output index
//     }

//     pub fn to_bytes(&self) -> Vec<u8> {
//         // TODO: Serialize as: txid (32 bytes) + vout (4 bytes, little-endian)
//     }

//     pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
//         // TODO: Deserialize 36 bytes: txid[0..32], vout[32..36]
//         // Return error if insufficient bytes
//     }
// }

// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// pub struct Script {
//     pub bytes: Vec<u8>,
// }

// impl Script {
//     pub fn new(bytes: Vec<u8>) -> Self {
//         // TODO: Simple constructor
//     }

//     pub fn to_bytes(&self) -> Vec<u8> {
//         // TODO: Prefix with CompactSize (length), then raw bytes
//     }

//     pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
//         // TODO: Parse CompactSize prefix, then read that many bytes
//         // Return error if not enough bytes
//     }
// }

// impl Deref for Script {
//     type Target = Vec<u8>;
//     fn deref(&self) -> &Self::Target {
//         // TODO: Allow &Script to be used as &[u8]
//     }
// }

// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// pub struct TransactionInput {
//     pub previous_output: OutPoint,
//     pub script_sig: Script,
//     pub sequence: u32,
// }

// impl TransactionInput {
//     pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
//         // TODO: Basic constructor
//     }

//     pub fn to_bytes(&self) -> Vec<u8> {
//         // TODO: Serialize: OutPoint + Script (with CompactSize) + sequence (4 bytes LE)
//     }

//     pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
//         // TODO: Deserialize in order:
//         // - OutPoint (36 bytes)
//         // - Script (with CompactSize)
//         // - Sequence (4 bytes)
//     }
// }

// #[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
// pub struct BitcoinTransaction {
//     pub version: u32,
//     pub inputs: Vec<TransactionInput>,
//     pub lock_time: u32,
// }

// impl BitcoinTransaction {
//     pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
//         // TODO: Construct a transaction from parts
//     }

//     pub fn to_bytes(&self) -> Vec<u8> {
//         // TODO: Format:
//         // - version (4 bytes LE)
//         // - CompactSize (number of inputs)
//         // - each input serialized
//         // - lock_time (4 bytes LE)
//     }

//     pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
//         // TODO: Read version, CompactSize for input count
//         // Parse inputs one by one
//         // Read final 4 bytes for lock_time
//     }
// }

// impl fmt::Display for BitcoinTransaction {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // TODO: Format a user-friendly string showing version, inputs, lock_time
//         // Display scriptSig length and bytes, and previous output info
//     }
// }
