# Chalamet

An implementation of the Chalamet Private Information Retrieval scheme.

We introduce **ChalametPIR**: a single-server Private Information Retrieval (PIR) scheme supporting fast, low-bandwidth keyword queries, with a conceptually very simple design. In particular, we develop a generic framework for converting PIR schemes for index queries over flat arrays (based on the Learning With Errors problem) into keyword PIR. This involves representing a key-value map using any probabilistic filter that permits reconstruction of elements from inclusion queries (e.g. Cuckoo filters). In particular, we make use of recently developed Binary Fuse filters to construct ChalametPIR, with minimal efficiency blow-up compared with state-of-the-art index-based schemes (all costs bounded by a factor of ≤ 1.08). Furthermore, we show that ChalametPIR achieves runtimes and financial costs that are factors of between 6×-11× and
3.75×-11.4× more efficient, respectively, than state-of-the-art keyword PIR approaches, for varying database configurations. Bandwidth costs are additionally reduced or remain competitive, depending on the configuration. Finally, we believe that our application of Binary Fuse filters in the cryptography setting may bring immediate independent value towards developing efficient variants of other related primitives that benefit from using such filters.

*Warning*: This code is a research prototype. Do not use it in production.

## Requirements

In order to natively build, run, test and benchmark the library, you will need the following:

```
  Rust >= 1.61.0
  Cargo
  Make
  Python3 >= 3.9.7
```

Note that we internally use the [xorf](https://github.com/ayazhafiz/xorf) library, but we modify it as seen [here](https://github.com/claucece/chalamet/tree/main/bff-modp).

To obtain our performance numbers as reported in Table 2 of our paper, we run our benchmarks in AWS EC2 ``t2.t2xlarge`` and ``c5.9xlarge`` machines, as reported.


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

#### Documentation

To view documentation (in a web browser manner):

```
  make docs
```

#### Benchmarking

There are several parameters that you can pass as flag to the `cargo bench` command, so that you can test the scheme.
These are (with their default values):

```
NUMBER_OF_ELEMENTS_EXP=16 (the m value of a DB: the number of rows)
LWE_DIMENSION=1774 (The LWE dimension)
ELEMENT_SIZE_BITS=8192 # 2**13 (the size of each element in bits)
PLAINTEXT_SIZE_EXP=10 (the size of each plaintext element: determines w of a DB: the size of the rows)
NUM_SHARDS=8
DB=true (if the offline steps will be bechmarked: these steps are very slow)

```

---

To run a simple benchmark (for a DB of 2^16 x 1024B) with offline steps, run (note the this process is slow. On average, it takes 12 minutes):

```
  make bench
```

This command will execute client query benchmarks and Database generation benchmarks (for more details, see the `benches/bench.rs` file).

---
To run all benchmarks  as reported in lines 1-10 of Table 2 of our paper (note that this process is very slow, it takes around 30 minutes):

```
  make bench-standard
```

This command will execute client query benchmarks and Database generation benchmarks for 16, 17, 18, 19 and 20 Number of DB items (log(m)). The results of these benchmarks can be found on Table 2 of our paper.

In order to see the results of the benchmarks, navigate to the `benchmarks-x-x.txt` file.

---
To run all benchmarks as reported in lines 11-13 of Table 2 of our paper and of Table 3 (note that this process is significantly slow):

```
  make bench-keyword-all
```

In order to see the results of the benchmarks, navigate to the `benchmarks-x-x.txt` file.

In order to make the results of lines 11-13 of Table 2 of our paper and of Table 3 of our paper easier to reproduce, we have made available these three commands:


```
  make bench-keyword-20
  make bench-keyword-14
  make bench-keyword-17
```

which omit any offline steps, and can be run independently for 2^20 x 256B, 2^17 x 30kB and 2^14 x 100kB.

---

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
