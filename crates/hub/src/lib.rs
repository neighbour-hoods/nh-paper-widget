use hdk::prelude::*;

// create_sensemaker_entry_full, get_sensemaker_entry,
// get_sensemaker_entry_by_headerhash, pack_ses_into_list_se,
// CreateSensemakerEntryInput, CreateSensemakerEntryInputParse, SchemeEntry,
use common::{
    create_sensemaker_entry_parse, mk_application_se, CreateSensemakerEntryInputParse,
    SensemakerEntry,
};

pub mod util;

entry_defs![
    Path::entry_def(),
    SensemakerEntry::entry_def()
];

pub const SM_DATA_TAG: &str = "sm_data";

// // do links even exist here anymore if the annotation holds on to the path?
// #[hdk_extern]
// fn link_to_sensemaker_entry(data_entry_hash: EntryHash, path: String) -> ExternResult<()> {

//   let sensemaker_entryhash = match get_sensemaker_entry(path.into())? {
//       None => Err(WasmError::Guest(
//           "sm_init is uninitialized for annotation".to_string(),
//       )),
//       Some((sensemaker_entryhash, _)) => Ok(sensemaker_entryhash),
//   }?;


//   let sm_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, ANN_TAG));
//   create_link(data_entry_hash.clone(), sensemaker_entryhash, sm_data_link_tag)?;
//   Ok(())
// }

// #[hdk_entry]
// fn initialize_sm_data(path_string: String, link_tag_string: String) -> ExternResult(()) {
//     // get current sm_init
    
//     // create sm_data at sm_init value

//     // link to end of given path with given link tag

// }

// #[hdk_extern]
// fn get_state_machine_data(
//     target_entryhash: EntryHash, 
//     opt_label: Option<String>) -> ExternResult<Vec<(EntryHash, SensemakerEntry)>> {
//     let label: String = match opt_label {
//         None => "".into(),
//         Some(lab) => lab
//     };
//     let state_machine_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, label));
//     let links = get_links(target_entryhash, Some(state_machine_data_link_tag))?;
//     let mut response: Vec<(EntryHash, SensemakerEntry)> = Vec::new();
//     for link in links {
//         let sensemaker_entryhash = link.target.clone();
//         let sensemaker_entry = util::try_get_and_convert(sensemaker_entryhash.clone(), GetOptions::latest())?;
//         ret.push((sensemaker_entryhash, sensemaker_entry));
//     }
//     Ok(response)
// }

#[hdk_extern]
fn get_sensemaker_entry_by_path((path_string, link_tag_string): (String, String)) -> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    match get_single_linked_entry(path_string, link_tag_string)? {
        Some(entryhash) => {
            let sensemaker_entry = util::try_get_and_convert(entryhash.clone(), GetOptions::content())?;
            Ok(Some((entryhash, sensemaker_entry)))
        }
        None => Ok(None),
    }
}

fn get_single_linked_entry(path_string: String, link_tag_string: String) -> ExternResult<Option<EntryHash>> {

    let path = Path::from(path_string);
    let links = get_links(
        path.path_entry_hash()?, 
        Some(LinkTag::new(link_tag_string)))?;
    match links.into_iter().max_by(|x, y| x.timestamp.cmp(&y.timestamp)) {
        None => Ok(None),
        Some(link) => Ok(Some(link.target.into_entry_hash().expect("Should be an entry.")))
    }
}

#[hdk_extern]
fn set_sensemaker_entry((
    path_string,
    link_tag_string,
    expr_str): (String, String, String)) -> ExternResult<()> {
    let (_, sensemaker_entry) = create_sensemaker_entry_parse(CreateSensemakerEntryInputParse {
        expr: expr_str,
        args: vec![],
    })?;
    let sensemaker_entryhash = hash_entry(sensemaker_entry)?;

    let path = Path::try_from(path_string.clone())?;
    path.ensure()?;
    let anchor_hash = path.path_entry_hash()?;
    create_link(
        anchor_hash, 
        sensemaker_entryhash,
        LinkType(0),
        LinkTag::new(link_tag_string))?;
    Ok(())
}

//   // TODO get links from PATH
//   let path = Path::try_from(path_string.clone())?;
//   path.ensure()?;

/// updates the link from the anchor to point to `entryhash`. will remove any existing links.
/// returns true if there were links which were "overwritten".
// fn set_entry_link(path: String, entryhash: EntryHash) -> ExternResult<bool> {
//     // TODO convert anchor to PATH
//     let anchor = anchor(anchor_type.clone(), anchor_text)?;
//     let link_tag = LinkTag::new(anchor_type);
//     let links = get_links(anchor.clone(), Some(link_tag.clone()))?;
//     let did_overwrite = !links.is_empty();
//     for link in links {
//         delete_link(link.create_link_hash)?;
//     }
//     create_link(anchor, eh, link_tag)?;
//     Ok(did_overwrite)
// }


/// for a given EntryHash, look for a state machine state linked to it with the label suffix
/// (link tag ~ `sm_data/$label`). look up the currently selected `sm_comp/$label` and apply that to
/// both the state entry, and the action. update the link off of `target_eh` s.t. it points to the
/// new state. this accomplishes "stepping" of the state machine.
#[hdk_extern]
fn step_sm((
        path_string,
        entry_hash,
        act) : (String, EntryHash, String)) -> ExternResult<()> {
    // path -> widget.paperz.annotationz => link tag -> sm_comp
    let sm_data_path: String = format!("{}.{}", path_string, entry_hash);
    // 1. get sm_data
    let (sm_data_eh, _sm_data_entry) = match get_sensemaker_entry_by_path((sm_data_path.clone(), "sm_data".into()))? {
        Some(pair) => Ok(pair),
        None => Err(WasmError::Guest("sm_data: invalid".into())),
    }?;

    // 2. get sm_comp
    let (sm_comp_eh, _sm_comp_entry) = match get_sensemaker_entry_by_path((path_string, "sm_comp".into()))? {
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
            LinkTag::new("sm_data"));
        debug!("create_link hh : {:?}", hh);
    }
    Ok(())
}
