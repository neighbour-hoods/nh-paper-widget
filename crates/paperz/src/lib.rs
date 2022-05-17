use hdk::prelude::*;

// create_sensemaker_entry_full, get_sensemaker_entry,
// get_sensemaker_entry_by_headerhash, pack_ses_into_list_se,
// CreateSensemakerEntryInput, CreateSensemakerEntryInputParse, SchemeEntry,
use common::{
    create_sensemaker_entry_parse, mk_application_se, CreateSensemakerEntryInputParse,
    SensemakerEntry,
};
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
    Path::entry_def(),
    Paper::entry_def(),
    Annotation::entry_def(),
    SensemakerEntry::entry_def()
];

#[hdk_entry]
pub struct Paper {
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
fn get_all_papers(_: ()) -> ExternResult<Vec<(EntryHash, Paper)>> {
    let paper_entry_links = get_links(paper_anchor()?, Some(LinkTag::new(PAPER_TAG)))?;
    let mut paperz: Vec<(EntryHash, Paper)> = Vec::new();
    for lnk in paper_entry_links {
        let res: ExternResult<(EntryHash, Paper)> = {
            let paper_eh = lnk.target;
            let (paper, _hh) =
                util::try_get_and_convert_with_hh(paper_eh.clone(), GetOptions::content())?;
            Ok((paper_eh, paper))
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
    // TODO abstract/generalize this
    let se_eh = match get_sm_init(ANN_TAG.into())? {
        None => Err(WasmError::Guest(
            "sm_init is uninitialized for annotation".to_string(),
        )),
        Some((se_eh, _se)) => Ok(se_eh),
    }?;

    let ann_hh = create_entry(&ann)?;
    let ann_eh = hash_entry(&ann)?;
    create_link(ann_anchor()?, ann_eh.clone(), LinkTag::new(ANN_TAG))?;
    create_link(ann.paper_ref, ann_eh.clone(), LinkTag::new(ANN_TAG))?;

    // TODO abstract/generalize this
    let sm_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, ANN_TAG));
    create_link(ann_eh.clone(), se_eh, sm_data_link_tag)?;
    Ok((ann_eh, ann_hh))
}

#[hdk_extern]
fn get_annotations_for_paper(paper_eh: EntryHash) -> ExternResult<Vec<(EntryHash, Annotation)>> {
    let mut ret = Vec::new();
    for lnk in get_links(paper_eh, Some(LinkTag::new(ANN_TAG)))? {
        let ann_eh = lnk.target;
        match util::try_get_and_convert(ann_eh.clone(), GetOptions::content()) {
            Ok(ann) => {
                ret.push((ann_eh, ann));
            }
            Err(err) => {
                debug!("get_annotations_for_paper: err: {}", err);
            }
        }
    }
    Ok(ret)
}

#[hdk_extern]
fn get_sm_data_for_eh(
    (target_eh, opt_label): (EntryHash, Option<String>),
) -> ExternResult<Vec<(EntryHash, SensemakerEntry)>> {
    let label: String = match opt_label {
        None => "".into(),
        Some(lab) => lab,
    };
    let sm_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, label));
    let links = get_links(target_eh, Some(sm_data_link_tag))?;
    let mut ret: Vec<(EntryHash, SensemakerEntry)> = Vec::new();
    for lnk in links {
        let se_eh = lnk.target.clone();
        let se = util::try_get_and_convert(se_eh.clone(), GetOptions::latest())?;
        ret.push((se_eh, se));
    }
    Ok(ret)
}

pub const SM_COMP_ANCHOR: &str = "sm_comp";
pub const SM_INIT_ANCHOR: &str = "sm_init";
pub const SM_DATA_TAG: &str = "sm_data";

#[hdk_extern]
fn get_sm_init(label: String) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    get_sm_se_eh(SM_INIT_ANCHOR.into(), label)
}

#[hdk_extern]
fn get_sm_comp(label: String) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    get_sm_se_eh(SM_COMP_ANCHOR.into(), label)
}

fn get_sm_se_eh(
    anchor_type: String,
    anchor_text: String,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    let opt_eh = get_single_linked_entry(anchor_type, anchor_text)?;
    match opt_eh {
        Some(eh) => {
            let se = util::try_get_and_convert(eh.clone(), GetOptions::content())?;
            Ok(Some((eh, se)))
        }
        None => Ok(None),
    }
}

fn get_single_linked_entry(
    anchor_type: String,
    anchor_text: String,
) -> ExternResult<Option<EntryHash>> {
    let links = get_links(
        anchor(anchor_type.clone(), anchor_text)?,
        Some(LinkTag::new(anchor_type)),
    )?;
    match &links[..] {
        [link] => Ok(Some(link.target.clone())),
        _ => Ok(None),
    }
}

#[hdk_extern]
/// set the sm_init state for the label to the `rep_lang` interpretation of `expr_str`
pub fn set_sm_init_se_eh((label, expr_str): (String, String)) -> ExternResult<bool> {
    set_sm_se_eh(SM_INIT_ANCHOR.into(), label, expr_str)
}

#[hdk_extern]
/// set the sm_comp state for the label to the `rep_lang` interpretation of `expr_str`
pub fn set_sm_comp_se_eh((label, expr_str): (String, String)) -> ExternResult<bool> {
    set_sm_se_eh(SM_COMP_ANCHOR.into(), label, expr_str)
}

fn set_sm_se_eh(anchor_type: String, anchor_text: String, expr_str: String) -> ExternResult<bool> {
    let (_se_hh, se) = create_sensemaker_entry_parse(CreateSensemakerEntryInputParse {
        expr: expr_str,
        args: vec![],
    })?;
    let se_eh = hash_entry(se)?;
    set_entry_link(anchor_type, anchor_text, se_eh)
}

/// updates the link from the anchor to point to `eh`. will remove any existing links.
/// returns true if there were links which were "overwritten".
fn set_entry_link(anchor_type: String, anchor_text: String, eh: EntryHash) -> ExternResult<bool> {
    let anchor = anchor(anchor_type.clone(), anchor_text)?;
    let link_tag = LinkTag::new(anchor_type);
    let links = get_links(anchor.clone(), Some(link_tag.clone()))?;
    let did_overwrite = !links.is_empty();
    for link in links {
        delete_link(link.create_link_hash)?;
    }
    create_link(anchor, eh, link_tag)?;
    Ok(did_overwrite)
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes)]
pub struct StepSmInput {
    target_eh: EntryHash,
    label: String,
    act: String,
}

/// for a given EntryHash, look for a state machine state linked to it with the label suffix
/// (link tag ~ `sm_data/$label`). look up the currently selected `sm_comp/$label` and apply that to
/// both the state entry, and the action. update the link off of `target_eh` s.t. it points to the
/// new state. this accomplishes "stepping" of the state machine.
#[hdk_extern]
fn step_sm(
    StepSmInput {
        target_eh,
        label,
        act,
    }: StepSmInput,
) -> ExternResult<()> {
    let sm_comp_eh = match get_sm_comp(label.clone())? {
        Some((eh, _se)) => Ok(eh),
        None => Err(WasmError::Guest("sm_comp: invalid".into())),
    }?;
    let sm_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, label));
    let sm_data_link = {
        let links = get_links(target_eh.clone(), Some(sm_data_link_tag.clone()))?;
        match &links[..] {
            [link] => Ok(link.clone()),
            _ => Err(WasmError::Guest(format!(
                "step_sm: multiple sm_data/{} links exist. there should only be one.",
                label
            ))),
        }
    }?;
    let sm_data_eh = sm_data_link.target;
    let sm_comp_hh = util::get_hh(sm_comp_eh, GetOptions::content())?;
    let sm_data_hh = util::get_hh(sm_data_eh, GetOptions::content())?;

    let (act_se_hh, _act_se) = create_sensemaker_entry_parse(CreateSensemakerEntryInputParse {
        expr: act,
        args: vec![],
    })?;

    let application_se = mk_application_se(vec![sm_comp_hh, sm_data_hh, act_se_hh])?;
    debug!("{:?}", application_se);
    let _application_se_hh = create_entry(&application_se)?;
    let application_se_eh = hash_entry(&application_se)?;
    debug!("{:?}", application_se_eh);

    {
        let hh = delete_link(sm_data_link.create_link_hash)?;
        debug!("delete_link hh : {:?}", hh);
    }
    {
        let hh = create_link(target_eh, application_se_eh, sm_data_link_tag)?;
        debug!("create_link hh : {:?}", hh);
    }
    Ok(())
}
