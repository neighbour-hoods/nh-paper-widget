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
    pub paper_ref: EntryHash, // should this be a HeaderHash?
    pub page_num: u64,
    pub paragraph_num: u64,
    pub what_it_says: String,
    pub what_it_should_say: String,
}

fn paper_anchor() -> ExternResult<EntryHash> {
    anchor("paperz".into(), "".into())
}

#[hdk_extern]
fn upload_paper(paper: Paper) -> ExternResult<HeaderHash> {
    debug!(
        "upload_paper: received input of length {}",
        paper.blob_str.len()
    );

    let paper_hh = create_entry(&paper)?;
    let paper_eh = hash_entry(&paper)?;
    create_link(paper_anchor()?, paper_eh, LinkTag::new(PAPER_TAG))?;

    Ok(paper_hh)
}

#[hdk_extern]
fn get_all_papers(_: ()) -> ExternResult<Vec<(Paper, EntryHash)>> {
    let paper_entry_links = get_links(paper_anchor()?, Some(LinkTag::new(PAPER_TAG)))?;
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

pub const ANN_TAG: &str = "annotationz";

fn ann_anchor() -> ExternResult<EntryHash> {
    anchor(ANN_TAG.into(), "".into())
}

#[hdk_extern]
fn create_annotation(ann: Annotation) -> ExternResult<(EntryHash, HeaderHash)> {
    let ann_hh = create_entry(&ann)?;
    let ann_eh = hash_entry(&ann)?;
    create_link(ann_anchor()?, ann_eh.clone(), LinkTag::new(ANN_TAG))?;

    // TODO abstract/generalize this
    match get_sm_init_se_eh(ANN_TAG.into())? {
        None => Err(WasmError::Guest(
            "sm_init is uninitialized for annotation".to_string(),
        )),
        Some(se_eh) => {
            create_link(ann_eh.clone(), se_eh, LinkTag::new(SM_DATA_TAG))?;
            Ok((ann_eh, ann_hh))
        }
    }
}

pub const SM_COMP_ANCHOR: &str = "sm_comp";
pub const SM_INIT_ANCHOR: &str = "sm_init";
pub const SM_DATA_TAG: &str = "sm_data";

fn get_sm_init_se_eh(label: String) -> ExternResult<Option<EntryHash>> {
    let sm_init = get_links(
        anchor(SM_INIT_ANCHOR.into(), label)?,
        Some(LinkTag::new(SM_INIT_ANCHOR)),
    )?;
    match &sm_init[..] {
        [sm_init_link] => Ok(Some(sm_init_link.target.clone())),
        _ => Ok(None),
    }
}

#[hdk_extern]
pub fn set_sm_init_se_eh((label, eh): (String, EntryHash)) -> ExternResult<bool> {
    set_entry_link(SM_INIT_ANCHOR.into(), label, eh)
}

#[hdk_extern]
pub fn set_sm_comp_se_eh((label, eh): (String, EntryHash)) -> ExternResult<bool> {
    set_entry_link(SM_COMP_ANCHOR.into(), label, eh)
}

fn set_entry_link(anchor_type: String, anchor_text: String, eh: EntryHash) -> ExternResult<bool> {
    let anchor = anchor(anchor_type.clone(), anchor_text)?;
    let link_tag = LinkTag::new(anchor_type);
    let links = get_links(anchor.clone(), Some(link_tag.clone()))?;
    match &links[..] {
        [] => {
            create_link(anchor, eh, link_tag)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

// for a given EntryHash, look for a state machine state linked to it with the label suffix
// (link tag ~ `sm_data/$label`). look up the currently selected `sm_comp/$label` and apply that to
// both the state entry, and the action. update the link off of `target_eh` s.t. it points to the
// new state. this accomplishes "stepping" of the state machine.
#[allow(dead_code)]
#[allow(unused_variables)]
fn step_sm(target_eh: EntryHash, label: String, act: String) -> ExternResult<bool> {
    todo!()
}
