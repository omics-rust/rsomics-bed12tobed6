use std::fs::File;
use std::io;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bed12tobed6::{Bed12ToBed6Opts, bed12tobed6};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// Split `BED12` features into discrete `BED6` intervals.
///
/// Reads a `BED12` file and expands each record into one `BED6` row per
/// block. The chrom, name, score, and strand fields are preserved.
#[derive(Parser, Debug)]
#[command(name = "rsomics-bed12tobed6", disable_help_flag = true)]
pub struct Cli {
    /// Input `BED12` file (use `-` for stdin).
    #[arg(short = 'i', long = "input", default_value = "-")]
    pub input: PathBuf,

    /// Replace the score field with the 1-based block number.
    #[arg(short = 'n', long = "block-num")]
    pub block_num_score: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        let opts = Bed12ToBed6Opts {
            block_num_score: self.block_num_score,
        };
        let stdout = io::stdout();
        let out = stdout.lock();
        if self.input == PathBuf::from("-") {
            bed12tobed6(io::stdin().lock(), &opts, out)
        } else {
            let f = File::open(&self.input).map_err(RsomicsError::Io)?;
            bed12tobed6(f, &opts, out)
        }
    }
}

pub const HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Split BED12 features into discrete BED6 intervals.",
    origin: Some(Origin {
        upstream: "bedtools",
        upstream_license: "GPL-2.0",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btq033"),
    }),
    usage_lines: &["-i <FILE> [OPTIONS]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('i'),
                long: "input",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: Some("-"),
                description: "Input BED12 file (default: stdin)",
                why_default: None,
            },
            FlagSpec {
                short: Some('n'),
                long: "block-num",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Replace score with 1-based block number",
                why_default: None,
            },
            FlagSpec {
                short: Some('h'),
                long: "help",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Show this help",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Expand exon blocks from a transcript BED12",
            command: "rsomics-bed12tobed6 -i transcripts.bed12",
        },
        Example {
            description: "Label blocks with their ordinal number as score",
            command: "rsomics-bed12tobed6 -i transcripts.bed12 -n",
        },
    ],
    json_result_schema_doc: None,
};

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
