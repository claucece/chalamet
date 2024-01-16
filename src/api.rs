//! The `api` module is the public entry point for all PIR operations.
use std::fs;
use std::str;

use core::marker::PhantomData;
use serde::{Deserialize, Serialize};

pub use crate::db::{BaseParams, CommonParams, DatabaseMatrix};
use crate::db::{FilterParams, KVDatabase, KVParams, KeyValue};
use crate::db::{IndexDatabase, IndexParams};

use crate::errors::{
  ErrorOverflownAdd, ErrorQueryParamsReused, ResultBoxedError,
};
pub use crate::utils::format::*;
use crate::utils::lwe::*;
use crate::utils::matrices::*;

/// A `Shard` is an instance of a database, where each row corresponds
/// to a single element, that has been preprocessed by the server.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shard {
  db: IndexDatabase,
  base_params: IndexParams,
}
impl Shard {
  /// Expects a JSON file of base64-encoded strings in file path. It also
  /// expects the lwe dimension, m (the number of DB elements), element size
  /// (in bytes) of the database elements, and plaintext bits.
  /// It will call the 'from_base64_strings' function to generate the database.
  pub fn from_json_file(
    file_path: &str,
    lwe_dim: usize,
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let file_contents: String =
      fs::read_to_string(file_path).unwrap().parse().unwrap();
    let elements: Vec<String> = serde_json::from_str(&file_contents).unwrap();
    Shard::from_base64_strings(&elements, lwe_dim, m, elem_size, plaintext_bits)
  }

  /// Expects an array of base64-encoded strings and converts into a
  /// database that can process client queries
  pub fn from_base64_strings(
    base64_strs: &[String],
    lwe_dim: usize,
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let db = IndexDatabase::new(base64_strs, m, elem_size, plaintext_bits)?;
    let base_params = IndexParams::new(&db, lwe_dim);
    Ok(Self { db, base_params })
  }

  /// Write base_params and DB to file
  pub fn write_to_file(
    &self,
    db_path: &str,
    params_path: &str,
  ) -> ResultBoxedError<()> {
    self.db.write_to_file(db_path)?;
    self.base_params.write_to_file(params_path)?;
    Ok(())
  }

  // Produces a serialized response (base64-encoded) to a serialized
  // client query
  pub fn respond(&self, q: &Query) -> ResultBoxedError<Vec<u8>> {
    let q = q.as_slice();
    let resp = Response(
      (0..self.db.get_row_width_self())
        .map(|i| self.db.vec_mult(q, i))
        .collect(),
    );
    let ser = bincode::serialize(&resp);

    Ok(ser?)
  }

  /// Returns the database
  pub fn get_db(&self) -> &IndexDatabase {
    &self.db
  }

  /// Returns the base parameters
  pub fn get_base_params(&self) -> &IndexParams {
    &self.base_params
  }

  pub fn into_row_iter(&self) -> std::vec::IntoIter<std::string::String> {
    (0..self.get_db().get_matrix_height())
      .map(|i| self.get_db().get_db_entry(i))
      .collect::<Vec<String>>()
      .into_iter()
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncodedKV {
  key: String,
  value: String,
}

/// A `KVShard` is an instance of a key-value database, where each row
/// corresponds to a single entry of multiple filter structures. The
/// mathematical interaction between client queries and the KV database
/// is the same as for standard databases.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KVShard {
  db: KVDatabase,
  base_params: KVParams,
}

impl KVShard {
  /// Create a new `KVShard` from ready-made `KeyValue` struct.
  pub fn new(
    kvs: &[KeyValue],
    lwe_dim: usize,
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let db = KVDatabase::new(kvs, m, elem_size, plaintext_bits)?;
    let &FilterParams {
      seed,
      segment_length,
      segment_length_mask,
      segment_count_length,
    } = db.get_filter_params();
    let base_params = KVParams::new(
      &db,
      lwe_dim,
      seed,
      segment_length,
      segment_length_mask,
      segment_count_length,
    );
    Ok(Self { db, base_params })
  }

  /// Expects a JSON file of base64-encoded strings in file path. It also
  /// expects the lwe dimension, m (the number of DB elements), element size
  /// (in bytes) of the database elements, and plaintext bits.
  /// It will call the 'from_base64_strings' function to generate the database.
  pub fn from_json_file(
    file_path: &str,
    lwe_dim: usize,
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let file_contents: String =
      fs::read_to_string(file_path).unwrap().parse().unwrap();
    let enc_kvs: Vec<EncodedKV> = serde_json::from_str(&file_contents).unwrap();
    let keys: Vec<String> = enc_kvs.iter().map(|e| e.key.clone()).collect();
    let values: Vec<String> = enc_kvs.iter().map(|e| e.value.clone()).collect();
    KVShard::from_base64_strings(
      &keys,
      &values,
      lwe_dim,
      m,
      elem_size,
      plaintext_bits,
    )
  }

  /// Expects an array of base64-encoded strings and converts into a
  /// database that can process client queries
  pub fn from_base64_strings(
    keys: &[String],
    values: &[String],
    lwe_dim: usize,
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let db = KVDatabase::from_base64_strings(
      keys,
      values,
      m,
      elem_size,
      plaintext_bits,
    )?;
    let &FilterParams {
      seed,
      segment_length,
      segment_length_mask,
      segment_count_length,
    } = db.get_filter_params();
    let base_params = KVParams::new(
      &db,
      lwe_dim,
      seed,
      segment_length,
      segment_length_mask,
      segment_count_length,
    );
    Ok(Self { db, base_params })
  }

  /// Write base_params and DB to file
  pub fn write_to_file(
    &self,
    db_path: &str,
    params_path: &str,
  ) -> ResultBoxedError<()> {
    self.db.write_to_file(db_path)?;
    self.base_params.write_to_file(params_path)?;
    Ok(())
  }

  // Produces a serialized response (base64-encoded) to a serialized
  // client query
  pub fn respond(&self, q: &Query) -> ResultBoxedError<Vec<u8>> {
    let resp = Response(
      (0..self.db.get_row_width_self())
        .map(|i| self.db.vec_mult(q.as_slice(), i))
        .collect(),
    );
    let se = bincode::serialize(&resp);

    Ok(se?)
  }

  /// Returns the database
  pub fn get_db(&self) -> &KVDatabase {
    &self.db
  }

  /// Returns the base parameters
  pub fn get_base_params(&self) -> &KVParams {
    &self.base_params
  }

  pub fn into_row_iter(&self) -> std::vec::IntoIter<std::string::String> {
    (0..self.get_db().get_matrix_height())
      .map(|i| self.get_db().get_db_entry(i))
      .collect::<Vec<String>>()
      .into_iter()
  }
}

/// The `QueryParams` struct is initialized to be used for a client
/// query.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryParams<DB, EP> {
  lhs: Vec<u32>,
  rhs: Vec<u32>,
  elem_size: usize,
  plaintext_bits: usize,
  db: PhantomData<DB>,
  extra_params: Option<EP>,
  pub used: bool,
}

impl QueryParams<IndexDatabase, EmptyAuxParams> {
  /// Generates `QueryParams` for a `Database` that is not KV
  fn new(cp: &CommonParams, params: &IndexParams) -> ResultBoxedError<Self> {
    let s = random_ternary_vector(params.get_dim());
    Ok(Self {
      lhs: cp.mult_left(&s)?,
      rhs: params.mult_right(&s)?,
      elem_size: params.get_elem_size(),
      plaintext_bits: params.get_plaintext_bits(),
      db: Default::default(),
      extra_params: None,
      used: false,
    })
  }

  /// Prepares a new client query based on an input row_inde that is a digit
  pub fn generate_query(
    &mut self,
    row_index: usize,
  ) -> ResultBoxedError<Query> {
    if self.used {
      return Err(Box::new(ErrorQueryParamsReused {}));
    }
    self.used = true;
    let query_indicator = get_rounding_factor(self.plaintext_bits);
    let mut lhs = Vec::new();
    lhs.clone_from(&self.lhs.clone());
    let (result, check) = lhs[row_index].overflowing_add(query_indicator);
    if !check {
      lhs[row_index] = result;
    } else {
      return Err(Box::new(ErrorOverflownAdd {}));
    }
    Ok(Query(lhs))
  }

  /// Parses the output as a row of u32 values
  pub fn parse_resp_as_row(&self, resp: &Response) -> Vec<u32> {
    // get parameters for rounding
    let rounding_factor = get_rounding_factor(self.plaintext_bits);
    let rounding_floor = get_rounding_floor(self.plaintext_bits);
    let plaintext_size = get_plaintext_size(self.plaintext_bits);

    // perform division and rounding
    (0..IndexDatabase::get_row_width(self.elem_size, self.plaintext_bits))
      .map(|i| {
        let left = resp.0[i];
        let right = self.rhs[i];
        let unscaled_res = left.wrapping_sub(right);
        let scaled_res = unscaled_res / rounding_factor;
        let scaled_rem = unscaled_res % rounding_factor;
        let mut rounded_res = scaled_res;
        if scaled_rem > rounding_floor {
          rounded_res += 1;
        }
        rounded_res % plaintext_size
      })
      .collect()
  }

  /// Parses the output as bytes
  pub fn parse_resp_as_bytes(&self, resp: &Response) -> Vec<u8> {
    let row = self.parse_resp_as_row(resp);
    bytes_from_u32_slice(&row, self.plaintext_bits, self.elem_size)
  }

  /// Parses the output as a base64-encoded string
  pub fn parse_resp_as_base64(&self, resp: &Response) -> String {
    let row = self.parse_resp_as_row(resp);
    base64_from_u32_slice(&row, self.plaintext_bits, self.elem_size)
  }
}
impl QueryParams<KVDatabase, FilterParams> {
  /// Generates `QueryParams` for a `Database` that is KV
  fn new(cp: &CommonParams, params: &KVParams) -> ResultBoxedError<Self> {
    let s = random_ternary_vector(params.get_dim());
    Ok(Self {
      lhs: cp.mult_left(&s)?,
      rhs: params.mult_right(&s)?,
      elem_size: params.get_elem_size(),
      plaintext_bits: params.get_plaintext_bits(),
      db: Default::default(),
      extra_params: Some(params.get_filter_params()),
      used: false,
    })
  }

  /// Prepares a new client query based on an input row_index that is a key
  pub fn generate_query(&mut self, key: &[u64; 4]) -> ResultBoxedError<Query> {
    if self.used {
      return Err(Box::new(ErrorQueryParamsReused {}));
    }
    self.used = true;
    let query_indicator = get_rounding_factor(self.plaintext_bits);
    let mut lhs = Vec::new();
    lhs.clone_from(&self.lhs.clone());
    if self.extra_params.is_none() {
      return Err("No filter parameters set for KV QueryParams".into());
    }
    let indices = self.extra_params.as_ref().unwrap().get_hash_evals(key);
    for row_index in indices {
      lhs[row_index] = lhs[row_index].wrapping_add(query_indicator);
    }
    Ok(Query(lhs))
  }

  /// Parses the output as a row of u32 values
  pub fn parse_resp_as_row(
    &self,
    resp: &Response,
    key: &[u64; 4],
  ) -> ResultBoxedError<Vec<u32>> {
    // get parameters for rounding
    let rounding_factor = get_rounding_factor(self.plaintext_bits);
    let rounding_floor = get_rounding_floor(self.plaintext_bits);
    let plaintext_size = get_plaintext_size(self.plaintext_bits);

    // check FilterParams exst
    if self.extra_params.is_none() {
      return Err("No filter parameters set for KV QueryParams".into());
    }
    let fp = self.extra_params.as_ref().unwrap();

    // perform division and rounding
    Ok(
      (0..KVDatabase::get_row_width(self.elem_size, self.plaintext_bits))
        .map(|i| {
          let left = resp.0[i];
          let right = self.rhs[i];
          let unscaled_res = left.wrapping_sub(right);
          let scaled_res = unscaled_res / rounding_factor;
          let scaled_rem = unscaled_res % rounding_factor;
          let mut rounded_res = scaled_res;
          if scaled_rem > rounding_floor {
            rounded_res += 1;
          }
          let masked = rounded_res;
          let unmasked = fp.unmask_value(masked, key, i as u64);
          unmasked % plaintext_size
        })
        .collect(),
    )
  }

  /// Parses the output as bytes
  pub fn parse_resp_as_bytes(
    &self,
    resp: &Response,
    key: &[u64; 4],
  ) -> ResultBoxedError<Vec<u8>> {
    let row = self.parse_resp_as_row(resp, key)?;
    Ok(bytes_from_u32_slice(
      &row,
      self.plaintext_bits,
      self.elem_size,
    ))
  }

  /// Parses the output as a base64-encoded string
  pub fn parse_resp_as_base64(
    &self,
    resp: &Response,
    key: &[u64; 4],
  ) -> ResultBoxedError<String> {
    let row = self.parse_resp_as_row(resp, key)?;
    Ok(base64_from_u32_slice(
      &row,
      self.plaintext_bits,
      self.elem_size,
    ))
  }
}

/// Returns `QueryParams` for an Index-based DB (`IndexDatabase`)
pub fn generate_index_query_params(
  cp: &CommonParams,
  params: &IndexParams,
) -> ResultBoxedError<QueryParams<IndexDatabase, EmptyAuxParams>> {
  QueryParams::<IndexDatabase, EmptyAuxParams>::new(cp, params)
}

/// Returns `QueryParams` for an KV-based DB (`KVDatabase`)
pub fn generate_kv_query_params(
  cp: &CommonParams,
  params: &KVParams,
) -> ResultBoxedError<QueryParams<KVDatabase, FilterParams>> {
  QueryParams::<KVDatabase, FilterParams>::new(cp, params)
}

/// The `Query` struct holds the necessary information encoded in
/// a client PIR query to the server DB for a particular `row_index`. It
/// provides methods for parsing server responses.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query(Vec<u32>);
impl Query {
  pub fn as_slice(&self) -> &[u32] {
    &self.0
  }
}

/// The `Response` object wraps a response from a single shard
#[derive(Clone, Serialize, Deserialize)]
pub struct Response(Vec<u32>);
impl Response {
  pub fn as_slice(&self) -> &[u32] {
    &self.0
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyAuxParams {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::db::FilterParams;
  use rand_core::{OsRng, RngCore};

  #[test]
  fn client_query_to_server_10_times() {
    let m = 2u32.pow(12) as usize;
    let elem_size = 2u32.pow(8) as usize;
    let plaintext_bits = 11usize;
    let lwe_dim = 512;
    let db_eles = generate_db_eles(m, (elem_size + 7) / 8);
    let shard = Shard::from_base64_strings(
      &db_eles,
      lwe_dim,
      m,
      elem_size,
      plaintext_bits,
    )
    .unwrap();
    let bp = shard.get_base_params();
    let cp = CommonParams::from(bp);
    #[allow(clippy::needless_range_loop)]
    for i in 0..10 {
      let mut qp =
        QueryParams::<IndexDatabase, EmptyAuxParams>::new(&cp, bp).unwrap();
      let q = qp.generate_query(i).unwrap();
      let d_resp = shard.respond(&q).unwrap();
      let resp: Response = bincode::deserialize(&d_resp).unwrap();
      let output = qp.parse_resp_as_base64(&resp);
      assert_eq!(output, db_eles[i]);
    }
  }

  #[test]
  fn client_kv_query_to_server_10_times() {
    let m = 2u32.pow(12) as usize;
    let elem_size = 2u32.pow(8) as usize;
    let plaintext_bits = 11usize;
    let lwe_dim = 512;
    let db_eles = generate_kv_db_elems(m, (elem_size + 7) / 8);
    let res: ResultBoxedError<Vec<KeyValue>> = db_eles
      .iter()
      .map(|e| {
        KeyValue::from_base64_strings(&e.0, &e.1, elem_size, plaintext_bits)
      })
      .collect::<Vec<ResultBoxedError<KeyValue>>>()
      .into_iter()
      .collect();
    let kvs = res.unwrap();
    let shard =
      KVShard::new(&kvs, lwe_dim, m, elem_size, plaintext_bits).unwrap();
    let bp = shard.get_base_params();

    // Compute the real values for testing purposes
    for kv in &kvs {
      let v: Vec<Vec<usize>> = (0..shard.get_db().entries.len())
        .map(|_| shard.get_db().get_filter_params().get_hash_evals(&kv.key))
        .collect();
      for (j, col) in v.iter().enumerate() {
        let masked = col.iter().fold(0u32, |acc, r| {
          acc.wrapping_add(shard.get_db().entries[j][*r])
        });
        let unmasked = shard
          .get_db()
          .get_filter_params()
          .unmask_value(masked, &kv.key, j as u64);
        let add_modp = unmasked % 2u32.pow(plaintext_bits as u32);
        assert_eq!(kv.value[j], add_modp);
      }
    }

    let cp = CommonParams::from(bp);

    #[allow(clippy::needless_range_loop)]
    for i in 0..10 {
      let mut qp =
        QueryParams::<KVDatabase, FilterParams>::new(&cp, bp).unwrap();
      let q = qp.generate_query(&kvs[i].key).unwrap();

      let d_resp = shard.respond(&q).unwrap();
      let resp: Response = bincode::deserialize(&d_resp).unwrap();

      let output = qp.parse_resp_as_row(&resp, &kvs[i].key).unwrap();
      assert_eq!(output, kvs[i].value);
    }
  }

  #[test]
  fn client_query_to_server_attempt_params_reuse() {
    let m = 2u32.pow(6) as usize;
    let elem_size = 2u32.pow(8) as usize;
    let plaintext_bits = 10usize;
    let lwe_dim = 512;
    let db_eles = generate_db_eles(m, (elem_size + 7) / 8);
    let shard = Shard::from_base64_strings(
      &db_eles,
      lwe_dim,
      m,
      elem_size,
      plaintext_bits,
    )
    .unwrap();
    let bp = shard.get_base_params();
    let cp = CommonParams::from(bp);
    let mut qp =
      QueryParams::<IndexDatabase, EmptyAuxParams>::new(&cp, bp).unwrap();
    // should be successful in generating a query
    let res_unused = qp.generate_query(0);
    assert!(res_unused.is_ok());

    // should be "used"
    assert!(qp.used);

    // should be successful in generating a query
    let res = qp.generate_query(0);
    assert!(res.is_err());
  }

  fn generate_db_eles(num_eles: usize, ele_byte_len: usize) -> Vec<String> {
    let mut eles = Vec::with_capacity(num_eles);
    for _ in 0..num_eles {
      let mut ele = vec![0u8; ele_byte_len];
      OsRng.fill_bytes(&mut ele);
      let ele_str = base64::encode(ele);
      eles.push(ele_str);
    }
    eles
  }

  fn generate_kv_db_elems(
    num_eles: usize,
    ele_byte_len: usize,
  ) -> Vec<(String, String)> {
    let mut v = Vec::with_capacity(num_eles);
    for _ in 0..num_eles {
      let mut key = vec![0u8; 32];
      let mut ele = vec![0u8; ele_byte_len];
      OsRng.fill_bytes(&mut key);
      OsRng.fill_bytes(&mut ele);
      let key_str = base64::encode(key);
      let ele_str = base64::encode(ele);
      v.push((key_str, ele_str));
    }
    v
  }
}
