use hdk::prelude::*;

pub const OWNER_TAG: &str = "memez_owner";

entry_defs![Meme::entry_def(), MemeRoot::entry_def()];

#[hdk_entry]
struct Meme {
    // encoded img payload
    img_str: String,
}

#[hdk_entry]
struct MemeRoot;

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    params_string: String,
}

#[hdk_extern]
fn upload_meme(params: Params) -> ExternResult<HeaderHash> {
    let Params {
        params_string: p_str,
    } = params;
    debug!("received input of length {}", p_str.len());

    create_meme_root_if_needed()?;

    let meme = Meme { img_str: p_str };
    let meme_hh = create_entry(&meme)?;
    let meme_eh = hash_entry(&meme)?;
    create_link(hash_entry(MemeRoot)?, meme_eh, LinkTag::new(OWNER_TAG))?;

    Ok(meme_hh)
}

// TODO figure out how to send a `()` from JS so we can call without fake Params arg
#[hdk_extern]
fn get_all_meme_strings(_: Params) -> ExternResult<Vec<String>> {
    let meme_entry_links = get_links(hash_entry(MemeRoot)?, None)?;
    let meme_strings: Vec<String> = meme_entry_links
        .into_iter()
        .map(|lnk| {
            let meme_eh = lnk.target;

            // retrieve `Meme` element, decode to entry
            let element = (match get(meme_eh.clone(), GetOptions::content())? {
                Some(el) => Ok(el),
                None => Err(WasmError::Guest(format!(
                    "could not dereference hash: {}",
                    meme_eh
                ))),
            })?;
            let meme: Meme = match element.into_inner().1.to_app_option()? {
                Some(m) => Ok(m),
                None => Err(WasmError::Guest(format!("non-present arg: {}", meme_eh))),
            }?;
            Ok(meme.img_str)
        })
        // TODO figure out if we should propagate or filter `Err`s.
        // there may exist non-Memes which could be linked to the MemeRoot?
        // although in principle that shouldn't occur, it's possible that it
        // could "crash" the system by making this section always Err.
        .collect::<ExternResult<Vec<String>>>()?;
    Ok(meme_strings)
}

/// returns true if created, false if already exists
fn create_meme_root_if_needed() -> ExternResult<bool> {
    match get(hash_entry(&MemeRoot)?, GetOptions::content())? {
        None => {
            let _hh = create_entry(&MemeRoot)?;
            Ok(true)
        }
        Some(_) => Ok(false),
    }
}
