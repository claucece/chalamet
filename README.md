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

These can also be found on the Makefile (lines 9-14).

---

To run a simple benchmark (for a DB of 2^16 x 1024B) with offline steps, run (note the this process is slow. On average, it takes 12 minutes):

```
  make bench
```

This command will execute client query benchmarks and Database generation benchmarks (for more details, see the `benches/bench.rs` file).

---
To run all benchmarks  as reported in lines 1-10 of Table 2 of our paper (note that this process is very slow, it takes around 30 minutes):

```
  make bench-keyword-standard
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

In order to run the benchmarks for Table 4 (index-based PIR with FrodoPIR), one can one:

```
  make bench-index-standard #For lines 1-10
  make bench-index-all #For lines 11-13
```

---

If all benches build and run correctly, you should see an `Finished ... benchmarks` under them.
We use [Criterion](https://bheisler.github.io/criterion.rs/book/index.html) for benchmarking.
If you want to see and have explanations of the benchmarks, you can locally open `target/criterion/report/index.html` in your browser.

**Note**: When running the benches, a warning might appear ``Warning: Unable to complete 10 samples in 100.0s. You may wish to increase target time to 486.6s.``. If you want to silence the warning, you can change line 30 of `benches/bech.rs` file to 500 or more. Note that this will make the running of benches slower.

In order to interpret the `benchmarks-x-x.txt` files, we provide some guidance here:


First, we have the initial lines describing the parameters for the benchmark.

```
[KV] Starting benches for keyword PIR.
[KV] Setting up DB for benchmarking. This might take a while...
[KV] The params are: m: 65536, lwe_dim: 1774, elem_size: 8192, plaintext-bits: 10
[KV] Are we benchmarking offline steps? true
```

These simply describe the LWE parameters for running the PIR interaction, note that the [KV] part here shows that we are running the keyword PIR benchmarks. This part can take a while, as the database and the public parameters are being generated for the interaction. It also states if we are running any offline steps, which can be omitted as they are significantly slow.

Once this setup has completed, we see the following.

```
[KV] Setup complete, starting benchmarks...
[KV] Filter Params: segment-len: 2048, segment-len-mask: 2047, segment-count-len: 73728
[KV] Starting client query benchmarks
```

This describes the individual filter parameters for the filters being used, and informs us that the benchmarks will now be computed for each piece of functionality.

Each individual benchmark is then displayed in the following way.

```
Benchmarking lwe/[KV] server response, lwe_dim: 1774, matrix_height: 77824, omega: 820: Collecting 100 samples in estimated 5.2698 s (300 iterations)
Benchmarking lwe/[KV] server response, lwe_dim: 1774, matrix_height: 77824, omega: 820: Analyzing
lwe/[KV] server response, lwe_dim: 1774, matrix_height: 77824, omega: 820
                        time:   [17.566 ms 17.670 ms 17.789 ms]
```

The middle time here is the average taken over the number of samples displayed. The name of the benchmark in this case is "server response", which took 17.67ms, and this is the value that we used in the paper.

In terms of Table 2, the key benchmarks are "create client query prepare" (Query), "server response" (Response), and "client parse server response" (Parsing), as these are the main online operations in the protocol.

### Tests

We have two big tests that the library executes:

1. `client_kv_query_to_server_10_times` test which executes the client-to-server functionality:
   the client asks for an item in the database (by its key) and the server is able to privately return it.
   The test asserts that the returned item is indeed the correct item in the database.
   It executes a for loop 10 times.
2. `client_query_to_server_attempt_params_reuse` test which executes the client-to-server
   functionality one time. It asserts that once parameters for a query are used, they
   are marked as so, and cannot be reused.
