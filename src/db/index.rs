use std::fs;
use std::io::BufReader;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::{BaseParams, DatabaseMatrix};
use crate::errors::ResultBoxedError;
use crate::utils::format::*;
use crate::utils::matrices::*;
use crate::utils::random::generate_seed;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexDatabase {
  entries: Vec<Vec<u32>>,
  m: usize,
  elem_size: usize,
  plaintext_bits: usize,
}
impl IndexDatabase {
  pub fn new(
    elements: &[String],
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    Ok(Self {
      entries: swap_matrix_fmt(&construct_rows(
        elements,
        m,
        elem_size,
        plaintext_bits,
      )?),
      m,
      elem_size,
      plaintext_bits,
    })
  }

  pub fn from_file(
    db_file: &str,
    m: usize,
    elem_size: usize,
    plaintext_bits: usize,
  ) -> ResultBoxedError<Self> {
    let file_contents: String = fs::read_to_string(db_file)?.parse()?;
    let elements: Vec<String> = serde_json::from_str(&file_contents)?;
    Self::new(&elements, m, elem_size, plaintext_bits)
  }
}
impl DatabaseMatrix for IndexDatabase {
  fn switch_fmt(&mut self) {
    self.entries = swap_matrix_fmt(&self.entries);
  }

  fn vec_mult(&self, row: &[u32], col_idx: usize) -> u32 {
    if row.len() != self.entries[col_idx].len() {
      panic!(
        "Incorrect multiplication, row_len: {}, col_len: {}",
        row.len(),
        self.entries[col_idx].len()
      );
    }
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
    IndexDatabase::get_row_width(
      self.get_elem_size(),
      self.get_plaintext_bits(),
    )
  }

  /// Get the matrix size
  fn get_matrix_height(&self) -> usize {
    self.m
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

/// The `BaseParams` object allows loading and interacting with params that
/// are used by the client for constructing queries
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexParams {
  dim: usize,
  m: usize,
  public_seed: [u8; 32],
  rhs: Vec<Vec<u32>>,
  elem_size: usize,
  plaintext_bits: usize,
}
impl IndexParams {
  pub fn new(db: &IndexDatabase, dim: usize) -> Self {
    let public_seed = generate_seed();
    Self {
      public_seed,
      rhs: Self::generate_params_rhs(db, public_seed, dim),
      dim,
      m: db.get_matrix_height(),
      elem_size: db.get_elem_size(),
      plaintext_bits: db.get_plaintext_bits(),
    }
  }

  /// Load params from a JSON file
  pub fn load(params_path: &str) -> ResultBoxedError<Self> {
    let reader = BufReader::new(fs::File::open(params_path)?);
    Ok(serde_json::from_reader(reader)?)
  }
}
impl BaseParams for IndexParams {
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

fn construct_row(
  element: &str,
  plaintext_bits: usize,
  row_width: usize,
) -> ResultBoxedError<Vec<u32>> {
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

fn construct_rows(
  elements: &[String],
  m: usize,
  elem_size: usize,
  plaintext_bits: usize,
) -> ResultBoxedError<Vec<Vec<u32>>> {
  let row_width = IndexDatabase::get_row_width(elem_size, plaintext_bits);

  let result = (0..m).map(|i| -> ResultBoxedError<Vec<u32>> {
    construct_row(&elements[i], plaintext_bits, row_width)
  });

  result.collect()
}
