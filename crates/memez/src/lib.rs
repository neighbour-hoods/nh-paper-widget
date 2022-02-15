use hdk::prelude::*;

use common::{
    create_interchange_entry_full, create_interchange_entry_parse,
    get_interchange_entries_which_unify, get_interchange_entry,
    get_interchange_entry_by_headerhash, mk_application_ie, pack_ies_into_list_ie,
    CreateInterchangeEntryInput, CreateInterchangeEntryInputParse, InterchangeEntry, SchemeEntry,
};
use rep_lang_core::{
    abstract_syntax::{Expr, Lit, PrimOp},
    app,
};
use rep_lang_runtime::{
    eval::{FlatValue, Value},
    infer::{normalize, unifies, InferState},
    types::{type_arr, type_int, type_list, type_pair, Scheme},
};

pub const MEME_TAG: &str = "memez_meme";
pub const REACTION_TAG: &str = "memez_reaction";

entry_defs![
    Meme::entry_def(),
    MemeRoot::entry_def(),
    InterchangeEntry::entry_def(),
    SchemeEntry::entry_def()
];

#[hdk_entry]
pub struct Meme {
    // encoded img payload
    pub img_str: String,
}

#[hdk_entry]
struct MemeRoot;

// for compat with JS
#[derive(Debug, Serialize, Deserialize)]
struct ScoredMeme {
    meme_string: String,
    opt_score: Option<i64>,
    eh: EntryHash,
}

#[hdk_extern]
fn upload_meme(img_str: String) -> ExternResult<HeaderHash> {
    debug!("received input of length {}", img_str.len());

    create_meme_root_if_needed()?;

    let meme = Meme { img_str };
    let meme_hh = create_entry(&meme)?;
    let meme_eh = hash_entry(&meme)?;
    create_link(hash_entry(MemeRoot)?, meme_eh, LinkTag::new(MEME_TAG))?;

    Ok(meme_hh)
}

#[hdk_extern]
fn get_all_meme_strings(score_comp_ie_hh: HeaderHash) -> ExternResult<Vec<ScoredMeme>> {
    // let score_comp_ie_hh = HeaderHash::try_from(score_comp_ie_hh_str).map_err(|err|
    //     WasmError::Guest(format!("err: {}", err))
    // )?;
    let (_eh, score_comp_ie) = get_interchange_entry_by_headerhash(score_comp_ie_hh.clone())?;

    // check IE scheme is right
    let () = check_schemes_unify(score_comp_sc(), score_comp_ie.output_scheme)?;

    let meme_entry_links = get_links(hash_entry(MemeRoot)?, Some(LinkTag::new(MEME_TAG)))?;
    let mut meme_strings: Vec<ScoredMeme> = Vec::new();
    for lnk in meme_entry_links {
        let res: ExternResult<ScoredMeme> = {
            let meme_eh = lnk.target;
            let (_hh, meme) = get_meme(meme_eh.clone())?;
            let opt_score =
                (score_meme(meme_eh.clone(), score_comp_ie_hh.clone())?).map(|(_hh, ie)| {
                    // "adapter" / "converter" should go here and clean up the API
                    match ie.output_flat_value {
                        FlatValue(Value::VInt(x)) => x,
                        _ => panic!("impossible: type inference broken"),
                    }
                });

            Ok(ScoredMeme {
                meme_string: meme.img_str,
                opt_score,
                eh: meme_eh,
            })
        };

        match res {
            Ok(sm) => meme_strings.push(sm),
            Err(err) => debug!("err in fetching ScoredMeme: {}", err),
        }
    }
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

fn score_meme(
    meme_eh: EntryHash,
    score_comp_ie_hh: HeaderHash,
) -> ExternResult<Option<(HeaderHash, InterchangeEntry)>> {
    let reaction_ie_links = get_links(meme_eh, Some(LinkTag::new(REACTION_TAG)))?;
    let mut reaction_ie_hh_s: Vec<HeaderHash> = vec![];
    for link in reaction_ie_links {
        match get_interchange_entry(link.target) {
            Ok((hh, ie)) => {
                let () = check_schemes_unify(reaction_sc(), ie.output_scheme)?;
                reaction_ie_hh_s.push(hh);
            }
            Err(err) => {
                debug!("get_interchange_entry: err: {}", err);
            }
        }
    }

    let reaction_list_ie = pack_ies_into_list_ie(reaction_ie_hh_s)?;
    debug!("reaction_list_ie: {:?}", reaction_list_ie);
    let reaction_list_ie_hh = create_entry(&reaction_list_ie)?;
    let score_comp_application_ie = mk_application_ie(vec![score_comp_ie_hh, reaction_list_ie_hh])?;

    let score_comp_application_ie_hh = create_entry(&score_comp_application_ie)?;

    Ok(Some((
        score_comp_application_ie_hh,
        score_comp_application_ie,
    )))
}

#[derive(Debug, Serialize, Deserialize)]
struct ReactToMemeInput {
    meme_eh: EntryHash,
    reaction_name: String,
    count: u32,
}

#[hdk_extern]
fn react_to_meme(rtmi: ReactToMemeInput) -> ExternResult<bool> {
    let opt_tag: Option<i64> = match rtmi.reaction_name.as_str() {
        "lulz" => Some(0),
        "mbz" => Some(1),
        // unknown reaction
        _ => None,
    };
    match opt_tag {
        // unknown reaction - return false
        None => Ok(false),
        Some(tag) => {
            let expr = app!(
                app!(Expr::Prim(PrimOp::Pair), Expr::Lit(Lit::LInt(tag))),
                Expr::Lit(Lit::LInt(rtmi.count.into()))
            );
            let (_ie_hh, ie_eh, _ie) =
                create_interchange_entry_full(CreateInterchangeEntryInput { expr, args: vec![] })?;
            let _link_hh = create_link(rtmi.meme_eh, ie_eh, LinkTag::new(REACTION_TAG));
            Ok(true)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ScoreComputation {
    expr_str: String,
    ie_hh: HeaderHash,
}

fn reaction_sc() -> Scheme {
    let target_ty = type_pair(type_int(), type_int());
    Scheme(Vec::new(), target_ty)
}

fn score_comp_sc() -> Scheme {
    let target_ty = type_arr(type_list(type_pair(type_int(), type_int())), type_int());
    Scheme(Vec::new(), target_ty)
}

#[hdk_extern]
fn get_score_computations(_: ()) -> ExternResult<Vec<ScoreComputation>> {
    let target_sc = score_comp_sc();
    let score_comps = get_interchange_entries_which_unify(Some(target_sc))?;

    Ok(score_comps
        .into_iter()
        .map(|(hh, ie)| {
            let expr_str = format!("{:?}", ie.operator);
            let ie_hh = hh;
            ScoreComputation { expr_str, ie_hh }
        })
        .collect())
}

fn check_schemes_unify(expected_sc: Scheme, actual_sc: Scheme) -> ExternResult<()> {
    let mut is = InferState::new();

    let Scheme(_, normalized_expected_ty) = normalize(&mut is, expected_sc.clone());

    // check unification of normalized type
    let Scheme(_, normalized_actual_ty) = normalize(&mut is, actual_sc.clone());
    // we are only interested in whether a type error occured
    if unifies(normalized_expected_ty, normalized_actual_ty).is_ok() {
        Ok(())
    } else {
        Err(WasmError::Guest(format!(
            "unification error: score comp has wrong type.\n\tactual: {:?}\n\texpected: {:?}",
            actual_sc, expected_sc,
        )))
    }
}

/// takes a string which should parse to a `rep_lang` Expr with type
///   List (Int, Int) -> Int
/// and returns a string representation of the `HeaderHash` of the created
/// InterchangeEntry which houses the score computation.
#[hdk_extern]
fn create_score_computation(comp: String) -> ExternResult<HeaderHash> {
    debug!("{}", comp);
    let input = CreateInterchangeEntryInputParse {
        expr: comp,
        args: vec![],
    };
    let (hh, ie) = create_interchange_entry_parse(input)?;

    // check IE scheme is right
    let () = check_schemes_unify(score_comp_sc(), ie.output_scheme)?;

    Ok(hh)
}

pub fn get_meme(arg_hash: EntryHash) -> ExternResult<(HeaderHash, Meme)> {
    let element = (match get(arg_hash.clone(), GetOptions::content())? {
        Some(el) => Ok(el),
        None => Err(WasmError::Guest(format!(
            "could not dereference arg: {}",
            arg_hash
        ))),
    })?;
    let hh = element.header_address().clone();
    match element.into_inner().1.to_app_option()? {
        Some(mm) => Ok((hh, mm)),
        None => Err(WasmError::Guest(format!("non-present arg: {}", arg_hash))),
    }
}
