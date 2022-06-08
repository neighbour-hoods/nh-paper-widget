use hdk::prelude::{holo_hash::DnaHash, *};

use common::{util, SensemakerEntry};

pub const PAPER_TAG: &str = "paperz_paper";
pub const ANN_TAG: &str = "annotationz";
pub const HUB_CELL_ID_TAG: &str = "hub_cell_id";
pub const HUB_ZOME_NAME: &str = "hub_main";

pub const ANNOTATIONZ_PATH: &str = "widget.paperz.annotation";
// TODO when hub merges with sensemaker, put these in common lib.rs
pub const SM_COMP_TAG: &str = "sm_comp";
pub const SM_INIT_TAG: &str = "sm_init";
pub const SM_DATA_TAG: &str = "sm_data";

entry_defs![
    Paper::entry_def(),
    Annotation::entry_def(),
    HubCellId::entry_def(),
    PathEntry::entry_def()
];

#[derive(Clone)]
#[hdk_entry]
pub struct HubCellId {
    // must include extension
    pub dna_hash: DnaHash,
    // encoded file bytes payload
    // getting an error here on get_paperz. Deserialize("invalid type: byte array, expected u8")
    pub agent_pubkey: AgentPubKey,
}

impl HubCellId {
    fn to_cell_id(self) -> CellId {
        CellId::new(self.dna_hash, self.agent_pubkey)
    }
}

#[hdk_entry]
pub struct Paper {
    // must include extension
    pub filename: String,
    // encoded file bytes payload
    // getting an error here on get_paperz. Deserialize("invalid type: byte array, expected u8")
    pub blob_str: String,
}

#[hdk_entry]
pub struct Annotation {
    pub paper_ref: EntryHash, // should this be a HeaderHash? probably
    pub page_num: u64,
    pub paragraph_num: u64,
    pub what_it_says: String,
    pub what_it_should_say: String,
}

fn hub_cell_id_anchor() -> ExternResult<EntryHash> {
    anchor("hub_cellId".into(), "".into())
}

fn paper_anchor() -> ExternResult<EntryHash> {
    anchor("paperz".into(), "".into())
}

#[hdk_extern]
fn set_hub_cell_id((dna_hash, agent_pubkey): (DnaHash, AgentPubKey)) -> ExternResult<HeaderHash> {
    let hub_cell_id: HubCellId = HubCellId {
        dna_hash,
        agent_pubkey,
    };
    let hub_cell_id_hh = create_entry(hub_cell_id.clone())?;
    let hub_cell_id_eh = hash_entry(hub_cell_id)?;
    create_link(
        hub_cell_id_anchor()?,
        hub_cell_id_eh,
        LinkType(0),
        LinkTag::new(HUB_CELL_ID_TAG),
    )?;

    Ok(hub_cell_id_hh)
}

#[hdk_extern]
fn get_hub_cell_id(_: ()) -> ExternResult<CellId> {
    debug!("Getting hub cellId...");
    match get_single_linked_entry()? {
        Some(entryhash) => {
            let hub_cell_id_entry: HubCellId =
                util::try_get_and_convert(entryhash.clone(), GetOptions::content())?;
            Ok(hub_cell_id_entry.to_cell_id())
        }
        None => Err(WasmError::Guest("get_hub_cell_id: no cell_id".into())),
    }
}

fn get_single_linked_entry() -> ExternResult<Option<EntryHash>> {
    let links = get_links(hub_cell_id_anchor()?, Some(LinkTag::new(HUB_CELL_ID_TAG)))?;
    match links
        .into_iter()
        .max_by(|x, y| x.timestamp.cmp(&y.timestamp))
    {
        None => Ok(None),
        Some(link) => Ok(Some(
            link.target.into_entry_hash().expect("Should be an entry."),
        )),
    }
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
    debug!("Getting all paperz...");
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
    debug!("Getting annotations");
    let mut annotations: Vec<(EntryHash, Annotation)> = Vec::new();
    debug!("Created empty vector");
    for link in get_links(paper_entry_hash, Some(LinkTag::new(ANN_TAG)))? {
        debug!("Here is a links: {:?}", link);
        let annotation_entry_hash = link.target.into_entry_hash().expect("should be an Entry.");
        match util::try_get_and_convert(annotation_entry_hash.clone(), GetOptions::content()) {
            Ok(annotation) => {
                debug!("Annotation: {:?}", annotation);
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

    let cell_id = get_hub_cell_id(())?;
    call(
        CallTargetCell::Other(cell_id),
        HUB_ZOME_NAME.into(),
        "initialize_sm_data".into(),
        None,
        (
            "widget.paperz.annotationz".to_string(),
            annotation_entryhash.clone(),
        ),
    )?;

    Ok((annotation_entryhash, annotation_headerhash))
}

/**
* What is a Vec of (EH, SE) tuples?
*/
#[hdk_extern]
fn get_state_machine_data(
    target_eh: EntryHash,
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    let path_string = format!("widget.paperz.annotationz.{}", target_eh);
    get_state_machine_generic(path_string, SM_DATA_TAG.to_string())
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
    let cell_id = get_hub_cell_id(())?;
    match call(
        CallTargetCell::Other(cell_id),
        HUB_ZOME_NAME.into(),
        "get_sensemaker_entry_by_path".into(),
        None,
        (path_string, link_tag_string),
    )? {
        ZomeCallResponse::Ok(data) => {
            return Ok(data.decode()?);
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
    set_sensemaker_entry(path_string.into(), SM_INIT_TAG.into(), expr_str)
}

#[hdk_extern]
/// set the sm_comp state for the path_string to the `rep_lang` interpretation of `expr_str`
pub fn set_state_machine_comp((path_string, expr_str): (String, String)) -> ExternResult<bool> {
    set_sensemaker_entry(path_string.into(), SM_COMP_TAG.into(), expr_str)
}

fn set_sensemaker_entry(
    path_string: String,
    link_tag_string: String,
    expr_str: String,
) -> ExternResult<bool> {
    let cell_id = get_hub_cell_id(())?;
    match call(
        CallTargetCell::Other(cell_id),
        HUB_ZOME_NAME.into(),
        "set_sensemaker_entry_parse_rl_expr".into(),
        None,
        (path_string, link_tag_string, expr_str),
    )? {
        ZomeCallResponse::Ok(_) => return Ok(true),
        err => {
            error!("ZomeCallResponse error: {:?}", err);
            Err(WasmError::Guest(format!("set_sensemaker_entry: {:?}", err)))
        }
    }
}

#[hdk_extern]
fn step_sm((path_string, entry_hash, act): (String, EntryHash, String)) -> ExternResult<()> {
    let cell_id = get_hub_cell_id(())?;
    match call(
        CallTargetCell::Other(cell_id),
        HUB_ZOME_NAME.into(),
        "step_sm".into(),
        None,
        (path_string, entry_hash, act),
    )? {
        ZomeCallResponse::Ok(_) => return Ok(()),
        err => {
            error!("ZomeCallResponse error: {:?}", err);
            Err(WasmError::Guest(format!("step_sm: {:?}", err)))
        }
    }
}
