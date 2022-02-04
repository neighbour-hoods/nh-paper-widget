use hdk::prelude::*;

use common::{
    create_interchange_entry_parse, get_linked_interchange_entries_which_unify, mk_application_ie,
    pack_ies_into_list_ie, CreateInterchangeEntryInputParse, InterchangeEntry, SchemeEntry,
};
use rep_lang_runtime::{
    eval::{FlatValue, Value},
    infer::{normalize, unifies, InferState},
    types::{type_arr, type_int, type_pair, Scheme},
};

pub const OWNER_TAG: &str = "memez_owner";

entry_defs![
    Meme::entry_def(),
    MemeRoot::entry_def(),
    ScoreComputationRoot::entry_def(),
    InterchangeEntry::entry_def(),
    SchemeEntry::entry_def()
];

#[hdk_entry]
struct Meme {
    // encoded img payload
    img_str: String,
}

#[hdk_entry]
struct MemeRoot;

#[hdk_entry]
struct ScoreComputationRoot;

#[derive(Debug, Serialize, Deserialize)]
struct Params {
    params_string: String,
}

// for compat with JS
#[derive(Debug, Serialize, Deserialize)]
struct ScoredMeme {
    meme_string: String,
    opt_score: Option<i64>,
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
fn get_all_meme_strings(_: Params) -> ExternResult<Vec<ScoredMeme>> {
    let meme_entry_links = get_links(hash_entry(MemeRoot)?, None)?;
    let meme_strings: Vec<ScoredMeme> = meme_entry_links
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
            let opt_score = (score_meme(meme_eh)?).map(|(_hh, ie)| {
                // "adapter" / "converter" should go here and clean up the API
                match ie.output_flat_value {
                    FlatValue(Value::VInt(x)) => x,
                    _ => panic!("impossible: type inference broken"),
                }
            });

            Ok(ScoredMeme {
                meme_string: meme.img_str,
                opt_score,
            })
        })
        // TODO figure out if we should propagate or filter `Err`s.
        // there may exist non-Memes which could be linked to the MemeRoot?
        // although in principle that shouldn't occur, it's possible that it
        // could "crash" the system by making this section always Err.
        .collect::<ExternResult<Vec<ScoredMeme>>>()?;
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

fn score_meme(meme_eh: EntryHash) -> ExternResult<Option<(HeaderHash, InterchangeEntry)>> {
    let ty = type_pair(type_int(), type_int());
    let sc = Scheme(Vec::new(), ty);

    let reaction_ies = get_linked_interchange_entries_which_unify((meme_eh, Some(sc)))?;

    let reaction_list_ie = pack_ies_into_list_ie(reaction_ies.into_iter().map(|t| t.0).collect())?;

    // in general, this should have either length 0 or 1
    // if it has length 0, then a selection has not been made, and we should error out
    // if it has length 1, that is our Score Computation.
    let score_comp_ie_links = get_links(hash_entry(ScoreComputationRoot)?, None)?;

    match &score_comp_ie_links[..] {
        // we have a score comp
        [score_comp_ie_link] => {
            let score_comp_ie_eh = score_comp_ie_link.target.clone();
            let score_comp_ie_hh: HeaderHash = {
                let element = (match get(score_comp_ie_eh.clone(), GetOptions::content())? {
                    Some(el) => Ok(el),
                    None => Err(WasmError::Guest(format!(
                        "could not dereference arg: {}",
                        score_comp_ie_eh
                    ))),
                })?;
                element.header_hashed().as_hash().clone()
            };

            let reaction_list_ie_hh = create_entry(&reaction_list_ie)?;
            let score_comp_application_ie =
                mk_application_ie(vec![score_comp_ie_hh, reaction_list_ie_hh])?;

            let score_comp_application_ie_hh = create_entry(&score_comp_application_ie)?;

            Ok(Some((
                score_comp_application_ie_hh,
                score_comp_application_ie,
            )))
        }

        // no selected score comp - we can't score.
        _ => Ok(None),
    }
}

/// takes a string which should parse to a `rep_lang` Expr with type
///   List (Int, Int) -> Int
/// and returns a string representation of the `HeaderHash` of the created
/// InterchangeEntry which houses the score computation.
#[hdk_extern]
fn create_score_computation(comp: String) -> ExternResult<String> {
    let input = CreateInterchangeEntryInputParse {
        expr: comp,
        args: vec![],
    };
    let (hh, ie) = create_interchange_entry_parse(input)?;

    // check IE scheme is right
    let () = {
        let mut is = InferState::new();

        let target_ty = type_arr(type_pair(type_int(), type_int()), type_int());
        let target_sc = Scheme(Vec::new(), target_ty);
        let Scheme(_, normalized_target_ty) = normalize(&mut is, target_sc.clone());

        // check unification of normalized type
        let Scheme(_, normalized_ie_ty) = normalize(&mut is, ie.output_scheme.clone());
        // we are only interested in whether a type error occured
        if unifies(normalized_target_ty.clone(), normalized_ie_ty).is_ok() {
            Ok(())
        } else {
            Err(WasmError::Guest(format!(
                "unification error: score comp has wrong type.\n\tactual: {:?}\n\texpected: {:?}",
                ie.output_scheme, target_sc,
            )))
        }
    }?;

    Ok(hh.to_string())
}
