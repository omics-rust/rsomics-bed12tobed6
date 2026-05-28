# rsomics-bed12tobed6

Split BED12 features into discrete BED6 intervals — a fast Rust reimplementation
of `bedtools bed12tobed6`.

## Usage

```
rsomics-bed12tobed6 -i transcripts.bed12
```

Options:
- `-i / --input <file>` — input BED12 file (default: stdin)
- `-n / --block-num` — replace score with 1-based block number

## Install

```
cargo install rsomics-bed12tobed6
```

## Origin

This crate is an independent Rust reimplementation of `bedtools bed12ToBed6` based on:
- Quinlan & Hall (2010). BEDTools: a flexible suite of utilities for comparing
  genomic features. Bioinformatics 26(6): 841–842. DOI: 10.1093/bioinformatics/btq033
- The BED12 format specification and black-box behavior testing against
  `bedtools bed12tobed6 2.31.1`

No source code from the BEDTools upstream was used as reference during implementation.
Test fixtures are independently generated.

License: MIT OR Apache-2.0.
Upstream credit: BEDTools <https://github.com/arq5x/bedtools2> (GPL-2.0).
