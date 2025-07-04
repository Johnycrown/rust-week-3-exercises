use hex;
use serde::de::{Deserializer, Error as DeError, Visitor};
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
    // Create a new CompactSize from a u64 value
    pub fn new(value: u64) -> Self {
        CompactSize { value }
    }

    // Convert the CompactSize to Bitcoin's variable-length encoding
    pub fn to_bytes(&self) -> Vec<u8> {
        let value = self.value;
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

        encoded
    }

    // Decode a CompactSize from a byte slice, returning the value and bytes consumed
    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        if bytes.is_empty() {
            return Err(BitcoinError::InsufficientBytes);
        }

        let first = bytes[0];

        match first {
            val @ 0x00..=0xFC => Ok((CompactSize::new(val as u64), 1)),

            0xFD => {
                if bytes.len() < 3 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let val = u16::from_le_bytes([bytes[1], bytes[2]]) as u64;
                Ok((CompactSize::new(val), 3))
            }

            0xFE => {
                if bytes.len() < 5 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let val = u32::from_le_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]) as u64;
                Ok((CompactSize::new(val), 5))
            }

            0xFF => {
                if bytes.len() < 9 {
                    return Err(BitcoinError::InsufficientBytes);
                }
                let val = u64::from_le_bytes([
                    bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
                ]);
                Ok((CompactSize::new(val), 9))
            }
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Txid(pub [u8; 32]);

impl Serialize for Txid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Reverse the byte order (Bitcoin txids are displayed in little-endian)
        let reversed: Vec<u8> = self.0.iter().rev().cloned().collect();

        // Encode the reversed bytes to hex
        let hex_string = hex::encode(reversed);

        // Serialize the hex string
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Txid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TxidVisitor;

        impl<'de> Visitor<'de> for TxidVisitor {
            type Value = Txid;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 64-character hex string representing a Txid")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                let bytes = hex::decode(v).map_err(DeError::custom)?;

                if bytes.len() != 32 {
                    return Err(DeError::custom("Txid must be exactly 32 bytes"));
                }

                // Bitcoin txids are serialized in little-endian, so reverse the byte order
                let mut reversed = [0u8; 32];
                reversed.copy_from_slice(&bytes.iter().rev().cloned().collect::<Vec<u8>>());
                Ok(Txid(reversed))
            }
        }

        deserializer.deserialize_str(TxidVisitor)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Txid,
    pub vout: u32,
}

impl OutPoint {
    pub fn new(txid: [u8; 32], vout: u32) -> Self {
        OutPoint {
            txid: Txid(txid),
            vout,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(36);
        let mut txid_bytes = self.txid.0;
        txid_bytes.reverse(); // Serialize txid in little-endian order
        bytes.extend_from_slice(&txid_bytes);
        bytes.extend_from_slice(&self.vout.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        if bytes.len() < 36 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let mut txid_bytes = [0u8; 32];
        txid_bytes.copy_from_slice(&bytes[..32]);
        txid_bytes.reverse(); // txid is stored in little-endian, reverse to internal format

        let mut vout_bytes = [0u8; 4];
        vout_bytes.copy_from_slice(&bytes[32..36]);
        let vout = u32::from_le_bytes(vout_bytes);

        Ok((OutPoint::new(txid_bytes, vout), 36))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Script {
    pub bytes: Vec<u8>,
}

impl Script {
    pub fn new(bytes: Vec<u8>) -> Self {
        Script { bytes }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let length_prefix = CompactSize::new(self.bytes.len() as u64).to_bytes();
        result.extend_from_slice(&length_prefix);
        result.extend_from_slice(&self.bytes);

        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        let (length_prefix, prefix_len) = CompactSize::from_bytes(bytes)?;
        let script_len = length_prefix.value as usize;

        let total_len = prefix_len + script_len;
        if bytes.len() < total_len {
            return Err(BitcoinError::InsufficientBytes);
        }

        let script_bytes = bytes[prefix_len..total_len].to_vec();
        Ok((Script::new(script_bytes), total_len))
    }
}

impl Deref for Script {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub script_sig: Script,
    pub sequence: u32,
}

impl TransactionInput {
    pub fn new(previous_output: OutPoint, script_sig: Script, sequence: u32) -> Self {
        TransactionInput {
            previous_output,
            script_sig,
            sequence,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.previous_output.to_bytes());
        result.extend_from_slice(&self.script_sig.to_bytes());
        result.extend_from_slice(&self.sequence.to_le_bytes());
        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        let (out_point, out_len) = OutPoint::from_bytes(bytes)?;
        let (script_sig, script_len) = Script::from_bytes(&bytes[out_len..])?;

        let offset = out_len + script_len;

        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }

        let mut seq_bytes = [0u8; 4];
        seq_bytes.copy_from_slice(&bytes[offset..offset + 4]);
        let sequence = u32::from_le_bytes(seq_bytes);

        Ok((
            TransactionInput {
                previous_output: out_point,
                script_sig,
                sequence,
            },
            offset + 4,
        ))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub lock_time: u32,
}
impl BitcoinTransaction {
    /// Constructs a Bitcoin transaction from version, inputs, and lock_time.
    pub fn new(version: u32, inputs: Vec<TransactionInput>, lock_time: u32) -> Self {
        BitcoinTransaction {
            version,
            inputs,
            lock_time,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&self.version.to_le_bytes());

        let input_count = CompactSize::new(self.inputs.len() as u64);
        result.extend_from_slice(&input_count.to_bytes());

        for input in &self.inputs {
            result.extend_from_slice(&input.to_bytes());
        }

        result.extend_from_slice(&self.lock_time.to_le_bytes());

        result
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<(Self, usize), BitcoinError> {
        let mut offset = 0;

        if bytes.len() < 4 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut ver_bytes = [0u8; 4];
        ver_bytes.copy_from_slice(&bytes[0..4]);
        let version = u32::from_le_bytes(ver_bytes);
        offset += 4;

        let (input_count_cs, input_count_len) = CompactSize::from_bytes(&bytes[offset..])?;
        let input_count = input_count_cs.value as usize;
        offset += input_count_len;

        let mut inputs = Vec::with_capacity(input_count);
        for _ in 0..input_count {
            let (input, input_len) = TransactionInput::from_bytes(&bytes[offset..])?;
            inputs.push(input);
            offset += input_len;
        }

        // Lock time
        if bytes.len() < offset + 4 {
            return Err(BitcoinError::InsufficientBytes);
        }
        let mut lt_bytes = [0u8; 4];
        lt_bytes.copy_from_slice(&bytes[offset..offset + 4]);
        let lock_time = u32::from_le_bytes(lt_bytes);
        offset += 4;

        Ok((
            BitcoinTransaction {
                version,
                inputs,
                lock_time,
            },
            offset,
        ))
    }
}

impl fmt::Display for BitcoinTransaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "BitcoinTransaction {{")?;
        writeln!(f, "  version: {}", self.version)?;
        writeln!(f, "  lock_time: {}", self.lock_time)?;
        writeln!(f, "  inputs [{}]:", self.inputs.len())?;
        for (i, input) in self.inputs.iter().enumerate() {
            // prev txid in human-readable hex (reverse back to big-endian for display)
            let mut be_txid = input.previous_output.txid.0;
            be_txid.reverse();
            let txid_hex = hex::encode(be_txid);

            // scriptSig as hex
            let script_hex = hex::encode(&*input.script_sig);

            writeln!(
                f,
                "    [{}] outpoint: {}:{} script_sig: {} sequence: {}",
                i, txid_hex, input.previous_output.vout, script_hex, input.sequence
            )?;
        }
        write!(f, "}}")
    }
}
