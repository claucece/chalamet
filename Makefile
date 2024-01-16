# DB location values
DB_FILE_NAME=rand_db.json
DB_FILE_PATH=data/${DB_FILE_NAME}
PARAMS_OUTPUT_PATH=data/params.json
PREVIOUS_DIR=..

# cargo build values
# these are the FrodoPIR parameters to be used for benchmarking
NUMBER_OF_ELEMENTS_EXP=16
LWE_DIMENSION=1774 # as per FrodoPIR paper
ELEMENT_SIZE_BITS=13
PLAINTEXT_SIZE_EXP=10
NUM_SHARDS=8

# rust flags
RUST_BACKTRACE=1

# python db generation values
DB_ALL_ONES=0
DB_NUM_ENTRIES_EXP=${NUMBER_OF_ELEMENTS_EXP}

RUST_FLAGS=RUST_BACKTRACE=${RUST_BACKTRACE}
DB_ENV=DB_FILE=${PREVIOUS_DIR}/${DB_FILE_PATH} PARAMS_OUTPUT_PATH=${PREVIOUS_DIR}/${PARAMS_OUTPUT_PATH}
DB_GEN_PRELIM=DB_ALL_ONES=${DB_ALL_ONES} DB_NUM_ENTRIES_EXP=${DB_NUM_ENTRIES_EXP} DB_OUTPUT_PATH=${DB_FILE_PATH} DB_ELEMENT_SIZE_BITS=${ELEMENT_SIZE_BITS}

PRELIM=${RUST_FLAGS}
PIR_FLAGS=-m ${NUMBER_OF_ELEMENTS_EXP} --dim ${LWE_DIMENSION} --ele_size ${ELEMENT_SIZE_BITS} --plaintext_bits ${PLAINTEXT_SIZE_EXP} --num_shards ${NUM_SHARDS}
PIR_ENV=PIR_NUMBER_OF_ELEMENTS_EXP=${NUMBER_OF_ELEMENTS_EXP} PIR_LWE_DIM=${LWE_DIMENSION} PIR_ELEM_SIZE_BITS=${ELEMENT_SIZE_BITS} PIR_PLAINTEXT_BITS=${PLAINTEXT_SIZE_EXP} PIR_NUM_SHARDS=${NUM_SHARDS}
PIR_ENV_ALL=PIR_LWE_DIM=${LWE_DIMENSION} PIR_NUM_SHARDS=${NUM_SHARDS}

LIB_PRELIM=${DB_FILE_PRELIM}
BIN_PRELIM=${BIN_DB_FILE_PRELIM} ${PARAMS_OUTPUT_PATH_PRELIM}

CARGO=cargo
CARGO_COMMAND=${PRELIM} ${CARGO}
PYTHON_COMMAND=${DB_GEN_PRELIM} python3

.PHONY: gen-db
gen-db:
	${PYTHON_COMMAND} data/generate_db.py

.PHONY: build test docs bench bench-all
build:
	${CARGO_COMMAND} build --release
test:
	${CARGO_COMMAND} test --release -- --nocapture

LOOPS = 100
test-loop:
	for ((i=1; i <= ${LOOPS}; ++i)) do make test && echo $$i || break; done

docs:
	${CARGO} doc --open --no-deps
bench:
	${PRELIM} ${PIR_ENV} ${CARGO} bench

bench-standard:
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=8192 PIR_NUMBER_OF_ELEMENTS_EXP=16 PIR_PLAINTEXT_BITS=10 ${CARGO} bench > benchmarks-16-1kb.txt
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=8192 PIR_NUMBER_OF_ELEMENTS_EXP=17 PIR_PLAINTEXT_BITS=10 ${CARGO} bench > benchmarks-17-1kb.txt
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=8192 PIR_NUMBER_OF_ELEMENTS_EXP=18 PIR_PLAINTEXT_BITS=10 ${CARGO} bench > benchmarks-18-1kb.txt
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=8192 PIR_NUMBER_OF_ELEMENTS_EXP=19 PIR_PLAINTEXT_BITS=9 ${CARGO} bench > benchmarks-19-1kb.txt
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=8192 PIR_NUMBER_OF_ELEMENTS_EXP=20 PIR_PLAINTEXT_BITS=9 ${CARGO} bench > benchmarks-20-1kb.txt

bench-keyword:
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=819200 PIR_NUMBER_OF_ELEMENTS_EXP=14 PIR_PLAINTEXT_BITS=10 ${CARGO} bench > benchmarks-14-kw.txt
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=245760 PIR_NUMBER_OF_ELEMENTS_EXP=17 PIR_PLAINTEXT_BITS=10 ${CARGO} bench > benchmarks-17-kw.txt
	${PRELIM} ${PIR_ENV_ALL} PIR_ELEM_SIZE_BITS=2048 PIR_NUMBER_OF_ELEMENTS_EXP=20 PIR_PLAINTEXT_BITS=9 ${CARGO} bench > benchmarks-20-kw.txt
