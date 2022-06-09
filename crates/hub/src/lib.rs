use hdk::prelude::*;

// create_sensemaker_entry_full, get_sensemaker_entry,
// get_sensemaker_entry_by_headerhash, pack_ses_into_list_se,
// CreateSensemakerEntryInput, CreateSensemakerEntryInputParse, SchemeEntry,
use common::{
    create_sensemaker_entry_parse, mk_application_se, util, CreateSensemakerEntryInputParse,
    SchemeEntry, SchemeRoot, SensemakerEntry, get_latest_path_entry,
};

entry_defs![
    Path::entry_def(),
    PathEntry::entry_def(),
    SensemakerEntry::entry_def(),
    SchemeEntry::entry_def(),
    SchemeRoot::entry_def()
];

pub const SM_INIT_TAG: &str = "sm_init";
pub const SM_DATA_TAG: &str = "sm_data";

#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
    let mut functions = GrantedFunctions::new();
    functions.insert((zome_info()?.name, "get_sensemaker_entry_by_path".into()));
    functions.insert((
        zome_info()?.name,
        "set_sensemaker_entry_parse_rl_expr".into(),
    ));
    functions.insert((zome_info()?.name, "initialize_sm_data".into()));

    let grant = ZomeCallCapGrant {
        access: CapAccess::Unrestricted,
        functions,
        tag: "".into(),
    };
    create_cap_grant(grant)?;

    Ok(InitCallbackResult::Pass)
}

#[hdk_extern]
fn get_sensemaker_entry_by_path(
    (path_string, link_tag_string): (String, String),
) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    match get_latest_path_entry(path_string, link_tag_string)? {
        Some(entryhash) => {
            let sensemaker_entry =
                util::try_get_and_convert(entryhash.clone(), GetOptions::content())?;
            Ok(Some((entryhash, sensemaker_entry)))
        }
        None => Ok(None),
    }
}

#[hdk_extern]
fn set_sensemaker_entry(
    (path_string, link_tag_string, target_eh): (String, String, EntryHash),
) -> ExternResult<()> {
    let path = Path::try_from(path_string)?;
    path.ensure()?;
    let anchor_hash = path.path_entry_hash()?;
    create_link(
        anchor_hash,
        target_eh,
        LinkType(0),
        LinkTag::new(link_tag_string),
    )?;
    Ok(())
}

#[hdk_extern]
fn set_sensemaker_entry_parse_rl_expr(
    (path_string, link_tag_string, expr_str): (String, String, String),
) -> ExternResult<()> {
    let (_, sensemaker_entry) = create_sensemaker_entry_parse(CreateSensemakerEntryInputParse {
        expr: expr_str,
        args: vec![],
    })?;
    let sensemaker_entryhash = hash_entry(sensemaker_entry)?;

    set_sensemaker_entry((path_string, link_tag_string, sensemaker_entryhash))
}

#[hdk_extern]
fn initialize_sm_data((path_string, target_eh): (String, EntryHash)) -> ExternResult<()> {
    let target_path_string = format!("{}.{}", path_string, target_eh);
    match get_latest_path_entry(path_string.clone(), SM_INIT_TAG.into())? {
        None => Err(WasmError::Guest("initialize_sm_data: no sm_init".into())),
        Some(init_eh) => set_sensemaker_entry((target_path_string, SM_DATA_TAG.into(), init_eh)),
    }
}

#[hdk_extern]
fn step_sm((path_string, entry_hash, act): (String, EntryHash, String)) -> ExternResult<()> {
    // path -> widget.paperz.annotationz => link tag -> sm_comp
    let sm_data_path: String = format!("{}.{}", path_string, entry_hash);
    // 1. get sm_data
    let (sm_data_eh, _sm_data_entry) =
        match get_sensemaker_entry_by_path((sm_data_path.clone(), "sm_data".into()))? {
            Some(pair) => Ok(pair),
            None => Err(WasmError::Guest("sm_data: invalid".into())),
        }?;

    // 2. get sm_comp
    let (sm_comp_eh, _sm_comp_entry) =
        match get_sensemaker_entry_by_path((path_string, "sm_comp".into()))? {
            Some(pair) => Ok(pair),
            None => Err(WasmError::Guest("sm_comp: invalid".into())),
        }?;

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
        let path = Path::from(sm_data_path);
        path.ensure()?;
        let path_hash = path.path_entry_hash()?;
        let hh = create_link(
            path_hash,
            application_se_eh,
            LinkType(0),
            LinkTag::new("sm_data"),
        );
        debug!("create_link hh : {:?}", hh);
    }
    Ok(())
}
