use std::fs;

use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::{BaseParams, DatabaseMatrix};

use crate::errors::ResultBoxedError;
use crate::utils::format::*;
use crate::utils::matrices::*;
use crate::utils::random::generate_seed;

use xorf::BinaryFuseP32;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyValue {
  pub key: [u64; 4],
  pub value: Vec<u32>,
}

impl KeyValue {
  pub fn from_base64_strings(
    k: &str,
    v: &str,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let key = sha256_into_u64_sized(k.as_bytes())?;
    let value = construct_row(v, plaintext_bits, elem_size)?;
    Ok(Self { key, value })
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct StorageFilters {
  filters: Vec<BinaryFuseP32>,
  seed: [u8; 32],
  segment_length: u32,
  segment_length_mask: u32,
  segment_count_length: u32,
}

impl StorageFilters {
  fn from_kvs(
    kvs: &[KeyValue],
    row_width: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<StorageFilters> {
    let keys: Vec<[u64; 4]> = kvs.iter().map(|kv| kv.key).collect();
    let mut seed = [0u8; 32];
    OsRng.fill_bytes(&mut seed);
    let filters: Vec<BinaryFuseP32> = (0..row_width)
      .map(|i| {
        let column: Vec<u32> = kvs.iter().map(|kv| kv.value[i]).collect();
        BinaryFuseP32::from_slice(
          seed,
          &keys,
          &column,
          i as u64,
          2u64.pow(plaintext_bits as u32),
        )
        .unwrap()
      })
      .collect();
    Ok(StorageFilters {
      filters: filters.clone(),
      seed,
      segment_length: filters[0].segment_length,
      segment_length_mask: filters[0].segment_length_mask,
      segment_count_length: filters[0].segment_count_length,
    })
  }

  fn get_columns(&self) -> Vec<Vec<u32>> {
    self
      .filters
      .iter()
      .map(|f| f.get_fingerprints_mod())
      .collect()
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilterParams {
  pub seed: [u8; 32],
  pub segment_length: u32,
  pub segment_length_mask: u32,
  pub segment_count_length: u32,
}
impl FilterParams {
  pub fn get_hash_evals(&self, key: &[u64; 4]) -> Vec<usize> {
    BinaryFuseP32::hash_eval(
      key,
      self.seed,
      self.segment_length,
      self.segment_length_mask,
      self.segment_count_length,
    )
  }

  fn get_key_fingerprint(&self, key: &[u64; 4], label: u64) -> u64 {
    BinaryFuseP32::get_key_fingerprint(key, self.seed, label)
  }

  pub fn unmask_value(&self, masked: u32, key: &[u64; 4], label: u64) -> u32 {
    masked.wrapping_add(self.get_key_fingerprint(key, label) as u32)
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KVDatabase {
  pub entries: Vec<Vec<u32>>,
  m: usize,
  elem_size: usize,
  plaintext_bits: usize,
  filter_params: FilterParams,
}

impl KVDatabase {
  pub fn new(
    kvs: &[KeyValue],
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let row_width = kvs[0].value.len();
    let filters = StorageFilters::from_kvs(kvs, row_width, plaintext_bits)?;

    Ok(Self {
      entries: filters.get_columns(),
      m,
      elem_size,
      plaintext_bits,
      filter_params: FilterParams {
        seed: filters.seed,
        segment_length: filters.segment_length,
        segment_length_mask: filters.segment_length_mask,
        segment_count_length: filters.segment_count_length,
      },
    })
  }

  pub fn from_base64_strings(
    keys: &[String],
    values: &[String],
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    if keys.len() != values.len() {
      return Err(
        format!(
          "Number of keys ({}) does not match number of values ({})",
          keys.len(),
          values.len()
        )
        .into(),
      );
    }
    let res: ResultBoxedError<Vec<KeyValue>> = (0..keys.len())
      .map(|i| {
        KeyValue::from_base64_strings(
          &keys[i],
          &values[i],
          elem_size,
          plaintext_bits,
        )
      })
      .collect::<Vec<ResultBoxedError<KeyValue>>>()
      .into_iter()
      .collect();
    if res.is_err() {
      return Err(
        format!("Error occurred constructing KVs: {:?}", res.err()).into(),
      );
    }
    let kvs = res.unwrap();
    KVDatabase::new(&kvs, m, elem_size, plaintext_bits)
  }

  pub fn get_filter_params(&self) -> &FilterParams {
    &self.filter_params
  }
}

impl DatabaseMatrix for KVDatabase {
  fn switch_fmt(&mut self) {
    self.entries = swap_matrix_fmt(&self.entries);
  }

  fn vec_mult(&self, row: &[u32], col_idx: usize) -> u32 {
    vec_mult_u32_u32(row, &self.entries[col_idx]).unwrap()
  }

  fn write_to_file(&self, path: &str) -> ResultBoxedError<()> {
    let json = json!(self.entries);
    Ok(serde_json::to_writer(&fs::File::create(path)?, &json)?)
  }

  /// Returns the ith row of the DB matrix
  fn get_row(&self, i: usize) -> Vec<u32> {
    self.entries[i].clone()
  }

  /// Returns the ith DB entry as a base64-encoded string
  fn get_db_entry(&self, i: usize) -> String {
    base64_from_u32_slice(
      &get_matrix_second_at(&self.entries, i),
      self.plaintext_bits,
      self.elem_size,
    )
  }

  /// Returns the width of each row in the DB matrix
  fn get_row_width(element_size: usize, plaintext_bits: usize) -> usize {
    let mut quo = element_size / plaintext_bits;
    if element_size % plaintext_bits != 0 {
      quo += 1;
    }
    quo
  }

  /// Returns the width of each row in the DB matrix
  fn get_row_width_self(&self) -> usize {
    KVDatabase::get_row_width(self.get_elem_size(), self.get_plaintext_bits())
  }

  /// Get the matrix size
  fn get_matrix_height(&self) -> usize {
    self.entries[0].len()
  }

  /// Get the element size
  fn get_elem_size(&self) -> usize {
    self.elem_size
  }

  /// Get the plaintext bits
  fn get_plaintext_bits(&self) -> usize {
    self.plaintext_bits
  }
}

fn construct_row(
  element: &str,
  plaintext_bits: usize,
  elem_size: usize,
) -> ResultBoxedError<Vec<u32>> {
  let row_width = KVDatabase::get_row_width(elem_size, plaintext_bits);
  let mut row = Vec::with_capacity(row_width);
  let bytes = base64::decode(element)?;
  let bits = bytes_to_bits_le(&bytes);
  for i in 0..row_width {
    let end_bound = (i + 1) * plaintext_bits;
    if end_bound < bits.len() {
      row.push(bits_to_u32_le(&bits[i * plaintext_bits..end_bound])?);
    } else {
      row.push(bits_to_u32_le(&bits[i * plaintext_bits..])?);
    }
  }
  Ok(row)
}

/// The `KVBaseParams` struct maintains additional functions compared
/// with `IndexParams`, for interacting with KV databases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVParams {
  dim: usize,
  m: usize,
  public_seed: [u8; 32],
  rhs: Vec<Vec<u32>>,
  elem_size: usize,
  plaintext_bits: usize,
  filter_params: FilterParams,
}
impl KVParams {
  pub fn new(
    db: &KVDatabase,
    dim: usize,
    seed: [u8; 32],
    segment_length: u32,
    segment_length_mask: u32,
    segment_count_length: u32,
  ) -> Self {
    let public_seed = generate_seed();
    Self {
      public_seed,
      rhs: Self::generate_params_rhs(db, public_seed, dim),
      dim,
      m: db.get_matrix_height(),
      elem_size: db.get_elem_size(),
      plaintext_bits: db.get_plaintext_bits(),
      filter_params: FilterParams {
        seed,
        segment_length,
        segment_length_mask,
        segment_count_length,
      },
    }
  }

  pub fn get_filter_params(&self) -> FilterParams {
    self.filter_params.clone()
  }
}
impl BaseParams for KVParams {
  fn get_total_records(&self) -> usize {
    self.m
  }

  fn get_dim(&self) -> usize {
    self.dim
  }

  fn get_elem_size(&self) -> usize {
    self.elem_size
  }

  fn get_plaintext_bits(&self) -> usize {
    self.plaintext_bits
  }

  fn get_public_seed(&self) -> [u8; 32] {
    self.public_seed
  }

  fn get_rhs(&self) -> &Vec<Vec<u32>> {
    &self.rhs
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check_consistent_retrieval() {
    let key = [1u64, 2, 3, 4];
    let value = vec![1u32, 2u32, 3u32];
    let row_width = value.len();
    let kv = KeyValue { key, value };
    let plaintext_bits = 10;
    let sfs =
      StorageFilters::from_kvs(&[kv.clone()], row_width, plaintext_bits)
        .unwrap();

    (0..row_width).for_each(|i| {
      assert!(sfs.filters[i].retrieve(&kv.key, i as u64) == kv.value[i])
    });
  }

  #[test]
  fn check_consistent_hashes() {
    let key = [1u64, 2, 3, 4];
    let value = vec![1u32, 2u32, 3u32];
    let row_width = value.len();
    let kv = KeyValue { key, value };
    let plaintext_bits = 10;
    let sfs =
      StorageFilters::from_kvs(&[kv.clone()], row_width, plaintext_bits)
        .unwrap();

    let v: Vec<Vec<usize>> = (0..row_width)
      .map(|_| {
        BinaryFuseP32::hash_eval(
          &kv.key,
          sfs.seed,
          sfs.segment_length,
          sfs.segment_length_mask,
          sfs.segment_count_length,
        )
      })
      .collect();
    for i in 1..row_width {
      assert_eq!(v[0], v[i]);
    }
  }

  #[test]
  fn db_check_consistent_retrieval() {
    let key = [1u64, 2, 3, 4];
    let value = vec![1u32, 2u32, 3u32];
    let len = value.len();
    let plaintext_bits = 10;
    let elem_size = plaintext_bits * len;
    let kv = KeyValue {
      key,
      value: value.clone(),
    };
    let kvdb =
      KVDatabase::new(&[kv.clone()], 1, elem_size, plaintext_bits).unwrap();

    let v: Vec<Vec<usize>> = (0..len)
      .map(|_| {
        BinaryFuseP32::hash_eval(
          &kv.key,
          kvdb.get_filter_params().seed,
          kvdb.get_filter_params().segment_length,
          kvdb.get_filter_params().segment_length_mask,
          kvdb.get_filter_params().segment_count_length,
        )
      })
      .collect();
    for i in 1..len {
      assert_eq!(v[0], v[i]);
    }
    for (i, col) in v.iter().enumerate() {
      let masked = col
        .iter()
        .fold(0u32, |acc, r| acc.wrapping_add(kvdb.entries[i][*r]));
      let unmasked = kvdb
        .get_filter_params()
        .unmask_value(masked, &key, i as u64);
      let add_modp = unmasked % 2u32.pow(plaintext_bits as u32);
      assert_eq!(add_modp, value[i]);
    }
  }

  #[test]
  fn attempt_actual_mult() {
    let key = [1u64, 2, 3, 4];
    let value = vec![1u32, 2u32, 3u32];
    let len = value.len();
    let plaintext_bits = 10;
    let elem_size = plaintext_bits * len;
    let kv = KeyValue {
      key,
      value: value.clone(),
    };
    let kvdb =
      KVDatabase::new(&[kv.clone()], 1, elem_size, plaintext_bits).unwrap();

    let mut mult_v = vec![0u32; kvdb.get_matrix_height()];
    let indices: Vec<Vec<usize>> = (0..len)
      .map(|_| {
        BinaryFuseP32::hash_eval(
          &kv.key,
          kvdb.get_filter_params().seed,
          kvdb.get_filter_params().segment_length,
          kvdb.get_filter_params().segment_length_mask,
          kvdb.get_filter_params().segment_count_length,
        )
      })
      .collect();
    for i in 1..len {
      assert_eq!(indices[0], indices[i]);
    }
    indices[0].iter().for_each(|idx| {
      mult_v[*idx] = 1;
    });
    for (i, y) in value.iter().enumerate().take(indices.len()) {
      let masked = kvdb.vec_mult(&mult_v, i);
      let unmasked = kvdb
        .get_filter_params()
        .unmask_value(masked, &key, i as u64);
      assert_eq!(unmasked % 2u32.pow(plaintext_bits as u32), *y);
    }
  }
}
