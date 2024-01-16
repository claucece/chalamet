# Chalamet

![Chalamet workflow](https://github.com/claucece/chalamet/actions/workflows/rust.yml/badge.svg)

An implementation of the Chalamet Private Information Retrieval scheme. Find the details over [our eprint paper](https://eprint.iacr.org/2022/981.pdf).

We introduce **ChalametPIR**: a single-server Private Information Retrieval (PIR) scheme supporting fast, low-bandwidth keyword queries, with a conceptually very simple design. In particular, we develop a generic framework for converting from PIR schemes for flat arrays (based on the Learning With Errors problem) into keyword PIR, by representing a key-value map using any data storage filter. In particular, we make use of recently developed Binary Fuse Filters to achieve a keyword PIR scheme with minimal blow-up (bounded by a factor of ≤ 1.08) compared with state-of-the-art index-based schemes. We implement ChalametPIR in Rust, and show that it achieves runtimes and financial costs that are factors of
between 6×-11× and 3.75×-11.4× more efficient, respectively, for varying database configurations. Bandwidth costs are either reduced or competitive depending on the configuration. While our focus is clearly on PIR, we believe that our application of Binary Fuse Filters may have independent value for other systems and cryptographic primitives.

*Warning*: This code is a research prototype. Do not use it in production.

## Requirements

In order to [natively](#native) build, run, test and benchmark the library, you will need the following:

```
  Rust >= 1.61.0
  Cargo
  Make
  Python3 >= 3.9.7
```

To obtain our performance numbers as reported in our paper, we run our benchmarks in AWS EC2 ``t2.t2xlarge`` and ``c5.9xlarge`` machines.

## Quickstart

### Local

#### Building

To install the latest version of Rust, use the following command (you can also check how to install on the [Rust documentation](https://www.rust-lang.org/tools/install)):

```
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

To build the library, run:

```
  make build
```

#### Testing

To run the tests:

```
  make test
```

We test:

* A client and server workflow when using Chalamet (internally with FrodoPIR) (10 times).
* A test to check that the library fails if parameters are reused.

If all test build and run correctly, you should see an `ok` next to them.

**Note**: Occasionally, one of the tests will fail with a `thread 'api::tests::client_query_to_server_10_times' panicked at 'assertion failed: (left == right)` error. This is due to the usage of specific parameters for testing and can be safely ignored.

#### Documentation

To view documentation (in a web browser manner):

```
  make docs
```

#### Benchmarking

To run a specific set of benchmarks, run (note the this process is slow. On average, it takes 12 minutes):

```
  make bench
```

This command will execute client query benchmarks and Database generation benchmarks (for more details, see the `benches/bench.rs` file).

To run all benchmarks (note that this process is very slow, it takes around 30 minutes):

```
  make bench-all
```

This command will execute client query benchmarks and Database generation benchmarks for 16, 17, 18, 19 and 20 Number of DB items (log(m)). The results of these benchmarks can be found on Table 2 of our paper.

In order to see the results of the benchmarks, navigate to the `benchmarks-x-x.txt` file.

If all benches build and run correctly, you should see an `Finished ... benchmarks` under them.
We use [Criterion](https://bheisler.github.io/criterion.rs/book/index.html) for benchmarking.
If you want to see and have explanations of the benchmarks, you can locally open `target/criterion/report/index.html` in your browser.

**Note**: When running the benches, a warning might appear ``Warning: Unable to complete 10 samples in 100.0s. You may wish to increase target time to 486.6s.``. If you want to silence the warning, you can change line 30 of `benches/bech.rs` file to 500 or more. Note that this will make the running of benches slower.

### Tests

We have two big tests that the library executes:

1. `client_kv_query_to_server_10_times` test which executes the client-to-server functionality:
   the client asks for an item in the database (by its key) and the server is able to privately return it.
   The test asserts that the returned item is indeed the correct item in the database.
   It executes a for loop 10 times.
2. `client_query_to_server_attempt_params_reuse` test which executes the client-to-server
   functionality one time. It asserts that once parameters for a query are used, they
   are marked as so, and cannot be reused.

## Citation

```
@misc{cryptoeprint:2024/949,
      author = {Sofía Celi and Alex Davidson},
      title = {Call Me By My Name: Simple, Practical Private Information Retrieval for Keyword Queries},
      howpublished = {Cryptology ePrint Archive, Paper 2022/981},
      year = {2024},
      note = {\url{https://eprint.iacr.org/2022/981}},
      url = {https://eprint.iacr.org/2022/981}
}
```
