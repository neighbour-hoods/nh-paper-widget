use hdk::prelude::*;

// create_sensemaker_entry_full, get_sensemaker_entry,
// get_sensemaker_entry_by_headerhash, pack_ses_into_list_se,
// CreateSensemakerEntryInput, CreateSensemakerEntryInputParse, SchemeEntry,
use common::{
    create_sensemaker_entry_parse, mk_application_se, CreateSensemakerEntryInputParse,
    SensemakerEntry,
};

mod util;

entry_defs![
    Path::entry_def(),
    SensemakerEntry::entry_def()
];

#[hdk_extern]
fn link_to_sensemaker_entry(data_entry_hash: EntryHash) -> ExternResult<()> {

  // TODO abstract/generalize this
  let sensemaker_entryhash = match get_state_machine(ANN_TAG.into())? {
      None => Err(WasmError::Guest(
          "sm_init is uninitialized for annotation".to_string(),
      )),
      Some((sensemaker_entryhash, _)) => Ok(sensemaker_entryhash),
  }?;

  // TODO abstract/generalize this
  let sm_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, ANN_TAG));
  create_link(data_entry_hash.clone(), sensemaker_entryhash, sm_data_link_tag)?;
  Ok(())
}

#[hdk_extern]
fn get_state_machine_data(
    target_entryhash: EntryHash, opt_label: Option<String>),
) -> ExternResult<Vec<(EntryHash, SensemakerEntry)>> {
    let label: String = match opt_label {
        None => "".into(),
        Some(lab) => lab,
    };
    let state_machine_data_link_tag = LinkTag::new(format!("{}/{}", SM_DATA_TAG, label));
    let links = get_links(target_entryhash, Some(state_machine_data_link_tag))?;
    let mut response: Vec<(EntryHash, SensemakerEntry)> = Vec::new();
    for link in links {
        let sensemaker_entryhash = link.target.clone();
        let sensemaker_entry = util::try_get_and_convert(sensemaker_entryhash.clone(), GetOptions::latest())?;
        ret.push((sensemaker_entryhash, sensemaker_entry));
    }
    Ok(response)
}
#[hdk_extern]
fn get_state_machine(path: String)-> ExternResult<Option<(EntryHash, SensemakerEntry)>> {
    let opt_entryhash = get_single_linked_entry(path)?;
    match opt_entryhash {
        Some(entryhash) => {
            let sensemaker_entry = util::try_get_and_convert(entryhash.clone(), GetOptions::content())?;
            Ok(Some((entryhash, sensemaker_entry)))
        }
        None => Ok(None),
    }
}

fn get_single_linked_entry(path: String) -> ExternResult<Option<EntryHash>> {
  // TODO get links from PATH
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
fn set_state_machine(path: String, expr_str: String) -> ExternResult<bool> {
    let (_, sensemaker_entry) = create_sensemaker_entry_parse(CreateSensemakerEntryInputParse {
        expr: expr_str,
        args: vec![],
    })?;
    let sensemaker_entryhash = hash_entry(sensemaker_entry)?;
    set_entry_link(path, sensemaker_entryhash)
}

/// updates the link from the anchor to point to `eh`. will remove any existing links.
/// returns true if there were links which were "overwritten".
fn set_entry_link(path: String, entryhash: EntryHash) -> ExternResult<bool> {
    // TODO convert anchor to PATH
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
    let sm_comp_eh = match get_state_machine(label.clone())? {
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
