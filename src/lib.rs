//! Split `BED12` features into discrete `BED6` intervals.
//!
//! Each `BED12` record encodes multiple blocks via `blockCount`, `blockSizes`,
//! and `blockStarts`. This crate expands each block into a separate `BED6`
//! row, preserving chrom, name, score, and strand.
//!
//! ## Reference
//!
//! `BEDTools` bed12ToBed6 — Quinlan & Hall (2010). Bioinformatics 26(6): 841–842.
//! DOI: 10.1093/bioinformatics/btq033

use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use rsomics_common::{Result, RsomicsError};

#[derive(Default)]
pub struct Bed12ToBed6Opts {
    /// Replace score with 1-based block number instead of the original score.
    pub block_num_score: bool,
}

pub fn bed12tobed6<W: Write>(reader: impl Read, opts: &Bed12ToBed6Opts, out: W) -> Result<()> {
    let mut writer = BufWriter::new(out);
    let buf = BufReader::new(reader);

    for line in buf.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let t = line.trim_end_matches(['\n', '\r']);
        if t.is_empty()
            || t.starts_with('#')
            || t.starts_with("track ")
            || t.starts_with("browser ")
        {
            continue;
        }
        process_record(t, opts, &mut writer)?;
    }

    writer.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

fn process_record(line: &str, opts: &Bed12ToBed6Opts, out: &mut impl Write) -> Result<()> {
    let fields: Vec<&str> = line.splitn(13, '\t').collect();
    if fields.len() < 12 {
        // Not a BED12 — emit as-is only if it has at least 3 fields (BED3)
        // but bed12tobed6 requires BED12 input; skip malformed rows.
        return Ok(());
    }

    let chrom = fields[0];
    let chrom_start: u64 = fields[1].parse().unwrap_or(0);
    let name = fields[3];
    let score = fields[4];
    let strand = fields[5];
    let block_count: usize = fields[9].parse().unwrap_or(0);

    // Parse block sizes and starts (comma-separated, may have trailing comma)
    let sizes: Vec<u64> = fields[10]
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    let starts: Vec<u64> = fields[11]
        .split(',')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    let n = block_count.min(sizes.len()).min(starts.len());
    // On minus-strand features, bedtools numbers blocks from the 3′ end
    // (rightmost block = block 1 in transcript coordinates).
    let minus = strand == "-";
    let mut ib = itoa::Buffer::new();

    for i in 0..n {
        let block_start = chrom_start + starts[i];
        let block_end = block_start + sizes[i];
        let score_field = if opts.block_num_score {
            let block_num = if minus { n - i } else { i + 1 };
            ib.format(block_num).to_string()
        } else {
            score.to_string()
        };

        out.write_all(chrom.as_bytes()).map_err(RsomicsError::Io)?;
        out.write_all(b"\t").map_err(RsomicsError::Io)?;
        out.write_all(ib.format(block_start).as_bytes())
            .map_err(RsomicsError::Io)?;
        out.write_all(b"\t").map_err(RsomicsError::Io)?;
        out.write_all(ib.format(block_end).as_bytes())
            .map_err(RsomicsError::Io)?;
        out.write_all(b"\t").map_err(RsomicsError::Io)?;
        out.write_all(name.as_bytes()).map_err(RsomicsError::Io)?;
        out.write_all(b"\t").map_err(RsomicsError::Io)?;
        out.write_all(score_field.as_bytes())
            .map_err(RsomicsError::Io)?;
        out.write_all(b"\t").map_err(RsomicsError::Io)?;
        out.write_all(strand.as_bytes()).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn run(input: &str, block_num_score: bool) -> String {
        let opts = Bed12ToBed6Opts { block_num_score };
        let mut out = Vec::new();
        bed12tobed6(Cursor::new(input), &opts, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn single_block() {
        // One block: start=0, size=100 → [100,200)
        let input = "chr1\t100\t200\tgene1\t0\t+\t100\t200\t0\t1\t100,\t0,\n";
        let out = run(input, false);
        assert_eq!(out.trim(), "chr1\t100\t200\tgene1\t0\t+");
    }

    #[test]
    fn two_blocks() {
        // Two exons: [100,200) and [300,400)
        // chrom_start=100, sizes=[100,100], starts=[0,200]
        let input = "chr1\t100\t400\tgene1\t255\t-\t100\t400\t0\t2\t100,100,\t0,200,\n";
        let out = run(input, false);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "chr1\t100\t200\tgene1\t255\t-");
        assert_eq!(lines[1], "chr1\t300\t400\tgene1\t255\t-");
    }

    #[test]
    fn block_num_score_flag() {
        let input = "chr1\t100\t400\tgene1\t255\t+\t100\t400\t0\t2\t100,100,\t0,200,\n";
        let out = run(input, true);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].ends_with("\t1\t+"), "first block score=1");
        assert!(lines[1].ends_with("\t2\t+"), "second block score=2");
    }

    #[test]
    fn skips_comment_and_track_lines() {
        let input = "# comment\ntrack name=foo\nchr1\t0\t100\tx\t0\t.\t0\t100\t0\t1\t100,\t0,\n";
        let out = run(input, false);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 1);
    }
}
