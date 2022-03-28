use hdk::prelude::*;

// create_sensemaker_entry_full, create_sensemaker_entry_parse, get_sensemaker_entry,
// get_sensemaker_entry_by_headerhash, mk_application_se, pack_ses_into_list_se,
// CreateSensemakerEntryInput, CreateSensemakerEntryInputParse, SchemeEntry,
use common::SensemakerEntry;
// use rep_lang_core::{
//     abstract_syntax::{Expr, Lit, PrimOp},
//     app, error,
// };
// use rep_lang_runtime::{
//     eval::{FlatValue, Value},
//     infer::{normalize, unifies, InferState},
//     types::{type_arr, type_int, type_list, type_pair, Scheme},
// };

mod util;

pub const PAPER_TAG: &str = "paperz_paper";
pub const REACTION_TAG: &str = "paperz_reaction";
pub const NAMED_SCORE_COMP_TAG: &str = "paperz_named_score_comp";

entry_defs![
    Paper::entry_def(),
    PaperRoot::entry_def(),
    SensemakerEntry::entry_def()
];

#[hdk_entry]
pub struct Paper {
    // human-readable title
    pub title: String,
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    pub blob_str: String,
}

#[hdk_entry]
struct PaperRoot;

#[hdk_extern]
fn upload_paper(paper: Paper) -> ExternResult<HeaderHash> {
    debug!(
        "upload_paper: received input of length {}",
        paper.blob_str.len()
    );

    create_paper_root_if_needed()?;

    let paper_hh = create_entry(&paper)?;
    let paper_eh = hash_entry(&paper)?;
    create_link(hash_entry(PaperRoot)?, paper_eh, LinkTag::new(PAPER_TAG))?;

    Ok(paper_hh)
}

#[hdk_extern]
fn get_all_papers(_: ()) -> ExternResult<Vec<(Paper, EntryHash)>> {
    let paper_entry_links = get_links(hash_entry(PaperRoot)?, Some(LinkTag::new(PAPER_TAG)))?;
    let mut paperz: Vec<(Paper, EntryHash)> = Vec::new();
    for lnk in paper_entry_links {
        let res: ExternResult<(Paper, EntryHash)> = {
            let paper_eh = lnk.target;
            let (paper, _hh) =
                util::try_get_and_convert_with_hh(paper_eh.clone(), GetOptions::content())?;
            Ok((paper, paper_eh))
        };

        match res {
            Ok(tup) => paperz.push(tup),
            Err(err) => debug!("err in fetching Paper: {}", err),
        }
    }
    Ok(paperz)
}

/// returns true if created, false if already exists
fn create_paper_root_if_needed() -> ExternResult<bool> {
    match get(hash_entry(&PaperRoot)?, GetOptions::content())? {
        None => {
            let _hh = create_entry(&PaperRoot)?;
            Ok(true)
        }
        Some(_) => Ok(false),
    }
}
