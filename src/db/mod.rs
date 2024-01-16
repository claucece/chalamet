mod index;
mod kv;

use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::errors::ResultBoxedError;
use crate::utils::matrices::*;

pub trait DatabaseMatrix {
  fn switch_fmt(&mut self);
  fn vec_mult(&self, row: &[u32], col_idx: usize) -> u32;
  fn write_to_file(&self, path: &str) -> ResultBoxedError<()>;
  fn get_row(&self, i: usize) -> Vec<u32>;
  fn get_db_entry(&self, i: usize) -> String;
  fn get_row_width(element_size: usize, plaintext_bits: usize) -> usize;
  fn get_row_width_self(&self) -> usize;
  fn get_matrix_height(&self) -> usize;
  fn get_elem_size(&self) -> usize;
  fn get_plaintext_bits(&self) -> usize;
}
pub use index::IndexDatabase;
pub use kv::KVDatabase;

pub trait BaseParams {
  /// Generates the RHS of the params using the database and the seed
  /// for the LHS
  fn generate_params_rhs<T: DatabaseMatrix>(
    db: &T,
    public_seed: [u8; 32],
    dim: usize,
  ) -> Vec<Vec<u32>> {
    let lhs = swap_matrix_fmt(&generate_lwe_matrix_from_seed(
      public_seed,
      dim,
      db.get_matrix_height(),
    ));
    (0..T::get_row_width(db.get_elem_size(), db.get_plaintext_bits()))
      .map(|i| {
        let mut col = Vec::with_capacity(db.get_matrix_height());
        for r in &lhs {
          col.push(db.vec_mult(r, i));
        }
        col
      })
      .collect()
  }
  /// Writes the params struct as JSON to file
  fn write_to_file(&self, path: &str) -> ResultBoxedError<()> {
    let json = json!({
      "lhs_seed": self.get_public_seed(),
      "rhs": self.get_rhs(),
    });
    Ok(serde_json::to_writer(&fs::File::create(path)?, &json)?)
  }
  /// Computes s*(A*DB) using the RHS of the public parameters
  fn mult_right(&self, s: &[u32]) -> ResultBoxedError<Vec<u32>> {
    let cols = self.get_rhs();
    (0..cols.len())
      .map(|i| vec_mult_u32_u32(s, &cols[i]))
      .collect()
  }
  fn get_total_records(&self) -> usize;
  fn get_dim(&self) -> usize;
  fn get_elem_size(&self) -> usize;
  fn get_plaintext_bits(&self) -> usize;
  fn get_public_seed(&self) -> [u8; 32];
  fn get_rhs(&self) -> &Vec<Vec<u32>>;
}
pub use index::IndexParams;
pub use kv::KVParams;

/// `CommonParams` holds the derived uniform matrix that is used for
/// constructing server public parameters and the client query.
#[derive(Serialize, Deserialize)]
pub struct CommonParams(Vec<Vec<u32>>);
impl CommonParams {
  // Returns the internal matrix
  pub fn as_matrix(&self) -> &[Vec<u32>] {
    &self.0
  }

  /// Computes s*A + e using the seed used to generate the LHS matrix of
  /// the public parameters
  pub fn mult_left(&self, s: &[u32]) -> ResultBoxedError<Vec<u32>> {
    let cols = self.as_matrix();
    (0..cols.len())
      .map(|i| {
        let s_a = vec_mult_u32_u32(s, &cols[i])?;
        let e = random_ternary();
        Ok(s_a.wrapping_add(e))
      })
      .collect()
  }
}
impl<T: BaseParams> From<&T> for CommonParams {
  fn from(params: &T) -> Self {
    Self(generate_lwe_matrix_from_seed(
      params.get_public_seed(),
      params.get_dim(),
      params.get_total_records(),
    ))
  }
}

pub use kv::FilterParams;
pub use kv::KeyValue;
