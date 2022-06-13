use hdk::prelude::{holo_hash::DnaHash, *};

use common::{
    get_latest_linked_entry, sensemaker_cell_id_anchor, sensemaker_cell_id_fns, util,
    SensemakerCellId, SensemakerEntry, OWNER_TAG, SENSEMAKER_ZOME_NAME, SM_COMP_TAG, SM_DATA_TAG,
    SM_INIT_TAG, compose_sensemaker_path, LINK_TYPE,
};

pub const PAPER_TAG: &str = "paperz_paper";
pub const ANN_TAG: &str = "annotationz";
pub const ANNOTATIONZ_PATH: &str = "widget.paperz.annotationz";

entry_defs![
    Paper::entry_def(),
    Annotation::entry_def(),
    SensemakerCellId::entry_def(),
    PathEntry::entry_def()
];

sensemaker_cell_id_fns! {}

#[hdk_entry]
pub struct Paper {
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    pub blob_str: String,
}

#[hdk_entry]
pub struct Annotation {
    pub paper_ref: EntryHash, // this should probably be a HeaderHash
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
    create_link(
        paper_anchor()?,
        paper_eh,
        LinkType(0),
        LinkTag::new(PAPER_TAG),
    )?;

    Ok(paper_hh)
}

#[hdk_extern]
fn get_all_paperz(_: ()) -> ExternResult<Vec<(EntryHash, Paper)>> {
    debug!("get_all_paperz: begin");
    let paper_entry_links = get_links(paper_anchor()?, Some(LinkTag::new(PAPER_TAG)))?;
    let mut paperz: Vec<(EntryHash, Paper)> = Vec::new();
    let mut opt_err = None;
    for lnk in paper_entry_links {
        let res: ExternResult<(EntryHash, Paper)> = {
            let paper_eh = lnk.target.into_entry_hash().expect("should be an Entry.");
            let (paper, _hh) =
                util::try_get_and_convert_with_hh(paper_eh.clone(), GetOptions::content())?;
            Ok((paper_eh, paper))
        };

        match res {
            Ok(tup) => paperz.push(tup),
            Err(err) => {
                debug!("err in fetching Paper: {}", err);
                opt_err = Some(err);
            }
        }
    }
    match opt_err {
        None => Ok(paperz),
        Some(err) => Err(WasmError::Guest(format!("get_all_paperz: {:?}", err))),
    }
}

fn annotation_anchor() -> ExternResult<EntryHash> {
    anchor(ANN_TAG.into(), "".into())
}

#[hdk_extern]
fn get_annotations_for_paper(
    paper_entry_hash: EntryHash,
) -> ExternResult<Vec<(EntryHash, Annotation)>> {
    debug!("get_annotations_for_paper: begin");
    let mut annotations: Vec<(EntryHash, Annotation)> = Vec::new();
    for link in get_links(paper_entry_hash, Some(LinkTag::new(ANN_TAG)))? {
        let annotation_entry_hash = link.target.into_entry_hash().expect("should be an Entry.");
        match util::try_get_and_convert(annotation_entry_hash.clone(), GetOptions::content()) {
            Ok(annotation) => {
                annotations.push((annotation_entry_hash, annotation));
            }
            Err(err) => {
                error!("get_annotations_for_paper: err: {}", err);
            }
        }
    }
    Ok(annotations)
}

#[hdk_extern]
fn create_annotation(annotation: Annotation) -> ExternResult<(EntryHash, HeaderHash)> {
    let annotation_headerhash = create_entry(&annotation)?;
    let annotation_entryhash = hash_entry(&annotation)?;
    create_link(
        annotation_anchor()?,
        annotation_entryhash.clone(),
        LinkType(0),
        LinkTag::new(ANN_TAG),
    )?;
    create_link(
        annotation.paper_ref,
        annotation_entryhash.clone(),
        LinkType(0),
        LinkTag::new(ANN_TAG),
    )?;

    let cell_id = get_sensemaker_cell_id(())?;
    call(
        CallTargetCell::Other(cell_id),
        SENSEMAKER_ZOME_NAME.into(),
        "initialize_sm_data".into(),
        None,
        (ANNOTATIONZ_PATH.to_string(), annotation_entryhash.clone()),
    )?;

    Ok((annotation_entryhash, annotation_headerhash))
}

#[hdk_extern]
fn get_state_machine_data(
    target_eh: EntryHash,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    let path_string = compose_sensemaker_path(&ANNOTATIONZ_PATH.to_string(), &target_eh);
    debug!("get_state_machine_data: path_string: {}", path_string);
    get_state_machine_generic(path_string, SM_DATA_TAG.into())
}

#[hdk_extern]
fn get_state_machine_init(
    path_string: String,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    get_state_machine_generic(path_string, SM_INIT_TAG.into())
}

#[hdk_extern]
fn get_state_machine_comp(
    path_string: String,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    get_state_machine_generic(path_string, SM_COMP_TAG.into())
}

fn get_state_machine_generic(
    path_string: String,
    link_tag_string: String,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    let cell_id = get_sensemaker_cell_id(())?;
    match call(
        CallTargetCell::Other(cell_id),
        SENSEMAKER_ZOME_NAME.into(),
        "get_sensemaker_entry_by_path".into(),
        None,
        (path_string, link_tag_string),
    )? {
        ZomeCallResponse::Ok(data) => {
            debug!("get_state_machine_generic: data: {:?}", data);
            Ok(data.decode()?)
        }
        err => {
            error!("ZomeCallResponse error: {:?}", err);
            Err(WasmError::Guest(format!(
                "get_state_machine_generic: {:?}",
                err
            )))
        }
    }
}

#[hdk_extern]
/// set the sm_init state for the path_string to the `rep_lang` interpretation of `expr_str`
pub fn set_state_machine_init((path_string, expr_str): (String, String)) -> ExternResult<bool> {
    set_sensemaker_entry(path_string, SM_INIT_TAG.into(), expr_str)
}

#[hdk_extern]
/// set the sm_comp state for the path_string to the `rep_lang` interpretation of `expr_str`
pub fn set_state_machine_comp((path_string, expr_str): (String, String)) -> ExternResult<bool> {
    set_sensemaker_entry(path_string, SM_COMP_TAG.into(), expr_str)
}

fn set_sensemaker_entry(
    path_string: String,
    link_tag_string: String,
    expr_str: String,
) -> ExternResult<bool> {
    let cell_id = get_sensemaker_cell_id(())?;
    match call(
        CallTargetCell::Other(cell_id),
        SENSEMAKER_ZOME_NAME.into(),
        "set_sensemaker_entry_parse_rl_expr".into(),
        None,
        (path_string, link_tag_string, expr_str),
    )? {
        ZomeCallResponse::Ok(_) => Ok(true),
        err => {
            error!("ZomeCallResponse error: {:?}", err);
            Err(WasmError::Guest(format!("set_sensemaker_entry: {:?}", err)))
        }
    }
}

#[hdk_extern]
fn step_sm_remote((path_string, entry_hash, act): (String, EntryHash, String)) -> ExternResult<()> {
    let cell_id = get_sensemaker_cell_id(())?;
    match call(
        CallTargetCell::Other(cell_id),
        SENSEMAKER_ZOME_NAME.into(),
        "step_sm".into(),
        None,
        (path_string, entry_hash, act),
    )? {
        ZomeCallResponse::Ok(_) => Ok(()),
        err => {
            error!("ZomeCallResponse error: {:?}", err);
            Err(WasmError::Guest(format!("step_sm: {:?}", err)))
        }
    }
}
