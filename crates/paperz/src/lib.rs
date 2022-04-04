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
    Annotation::entry_def(),
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
pub struct Annotation {
    pub page_num: u64,
    pub paragraph_num: u64,
    pub what_it_says: String,
    pub what_it_should_say: String,
}

#[hdk_extern]
fn upload_paper(paper: Paper) -> ExternResult<HeaderHash> {
    debug!(
        "upload_paper: received input of length {}",
        paper.blob_str.len()
    );

    let paper_hh = create_entry(&paper)?;
    let paper_eh = hash_entry(&paper)?;
    let paper_anchor = anchor("paperz".into(), "".into())?;
    create_link(paper_anchor, paper_eh, LinkTag::new(PAPER_TAG))?;

    Ok(paper_hh)
}

#[hdk_extern]
fn get_all_papers(_: ()) -> ExternResult<Vec<(Paper, EntryHash)>> {
    let paper_anchor = anchor("paperz".into(), "".into())?;
    let paper_entry_links = get_links(paper_anchor, Some(LinkTag::new(PAPER_TAG)))?;
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

pub const SM_COMP_ANCHOR: &str = "sm_comp";
pub const SM_INIT_ANCHOR: &str = "sm_init";
// this could be called `state`, but that is 5 letters instead of 4 and breaks symmetry.
pub const SM_DATA_ANCHOR: &str = "sm_data";
