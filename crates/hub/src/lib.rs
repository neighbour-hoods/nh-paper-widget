use hdk::prelude::*;

use common::{SchemeEntry, SchemeRoot, SensemakerEntry};

entry_defs![
    Path::entry_def(),
    PathEntry::entry_def(),
    SensemakerEntry::entry_def(),
    SchemeEntry::entry_def(),
    SchemeRoot::entry_def()
];

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
