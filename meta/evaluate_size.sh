#!/bin/bash

LANG=deen
TEST_OFFSET=50

for SIZE in $(seq 1 1 10); do 
     cargo run --release --bin slow_align -- \
          --file1 data/data_${LANG}_s --file2 data/data_${LANG}_t \
          --gold data/data_${LANG}_a --method search \
          --lowercase \
          --test-offset $TEST_OFFSET --dev-count $SIZE \
          1> /dev/null 2> /tmp/alignment_err_${LANG} & wait;
     TEST_AER=$(tail -n 2 /tmp/alignment_err_${LANG} | head -n 1 | sed "s/AER: //")
     TRAIN_AER=$(tail -n 5 /tmp/alignment_err_${LANG} | head -n 1 | sed "s/Best AER: //")
     echo $SIZE $TRAIN_AER $TEST_AER >> data/aers_size_${LANG};
done