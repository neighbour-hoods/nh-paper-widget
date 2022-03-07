use hdk::prelude::*;
use std::fmt::Debug;

use common::{
    create_sensemaker_entry_full, create_sensemaker_entry_parse, get_sensemaker_entry,
    get_sensemaker_entry_by_headerhash, mk_application_ie, pack_ies_into_list_ie,
    CreateSensemakerEntryInput, CreateSensemakerEntryInputParse, SchemeEntry, SensemakerEntry,
};
use rep_lang_core::{
    abstract_syntax::{Expr, Lit, PrimOp},
    app, error,
};
use rep_lang_runtime::{
    eval::{FlatValue, Value},
    infer::{normalize, unifies, InferState},
    types::{type_arr, type_int, type_list, type_pair, Scheme},
};

pub const MEME_TAG: &str = "memez_meme";
pub const REACTION_TAG: &str = "memez_reaction";
pub const NAMED_SCORE_COMP_TAG: &str = "memez_named_score_comp";

pub const AGGREGATOR_FN_COMP: &str = r#"
(let ([foldl
       (fix (lam [foldl]
         (lam [f acc xs]
           (if (null xs)
             acc
             (foldl
               f
               (f acc (head xs))
               (tail xs))))))]
      [folder
       (fix (lam [folder]
         (lam [acc tup]
           (if (null acc)
               (cons tup nil)
               (if (== (fst (head acc)) (fst tup))
                   (cons (pair (fst (head acc)) (+ (snd tup) (snd (head acc))))
                         (tail acc))
                   (cons (head acc)
                         (folder (tail acc) tup)))))))]
      [aggregator
       (lam [vals]
         (foldl folder nil vals))])
  aggregator)
"#;

entry_defs![
    Meme::entry_def(),
    MemeRoot::entry_def(),
    NamedScoreComputation::entry_def(),
    NamedScoreComputationRoot::entry_def(),
    SensemakerEntry::entry_def(),
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
    aggregated_reactions: Vec<(i64, i64)>,
    eh: EntryHash,
}

#[hdk_extern]
fn upload_meme(img_str: String) -> ExternResult<HeaderHash> {
    debug!("upload_meme: received input of length {}", img_str.len());

    create_meme_root_if_needed()?;

    let meme = Meme { img_str };
    let meme_hh = create_entry(&meme)?;
    let meme_eh = hash_entry(&meme)?;
    create_link(hash_entry(MemeRoot)?, meme_eh, LinkTag::new(MEME_TAG))?;

    Ok(meme_hh)
}

#[hdk_extern]
fn get_all_meme_strings(nsc_eh: EntryHash) -> ExternResult<Vec<ScoredMeme>> {
    // let score_comp_ie_hh = HeaderHash::try_from(score_comp_ie_hh_str).map_err(|err|
    //     WasmError::Guest(format!("err: {}", err))
    // )?;
    let (_nsc_hh, nsc) = get_nsc(nsc_eh)?;
    let (_eh, score_comp_ie) = get_sensemaker_entry_by_headerhash(nsc.score_comp_ie_hh.clone())?;

    // check IE scheme is right
    check_schemes_unify(score_comp_sc(), score_comp_ie.output_scheme)?;

    let meme_entry_links = get_links(hash_entry(MemeRoot)?, Some(LinkTag::new(MEME_TAG)))?;
    let mut meme_strings: Vec<ScoredMeme> = Vec::new();
    for lnk in meme_entry_links {
        let res: ExternResult<ScoredMeme> = {
            let meme_eh = lnk.target;
            let (_hh, meme) = get_meme(meme_eh.clone())?;
            let (_score_hh, score_ie, _aggregate_hh, aggregate_ie) =
                score_meme(meme_eh.clone(), nsc.score_comp_ie_hh.clone())?;
            // "adapter" / "converter" should go here and clean up the API
            let opt_score = match score_ie.output_flat_value {
                FlatValue(Value::VInt(x)) => Some(x),
                _ => error!("impossible: type inference broken"),
            };
            let aggregated_reactions = {
                let mut acc = vec![];
                vlist_of_pairs_to_vec_of_pairs(aggregate_ie.output_flat_value, &mut acc);
                acc
            };

            Ok(ScoredMeme {
                meme_string: meme.img_str,
                opt_score,
                aggregated_reactions,
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

fn vlist_of_pairs_to_vec_of_pairs<M: Debug>(ls: FlatValue<M>, acc: &mut Vec<(i64, i64)>) {
    match ls {
        FlatValue(Value::VCons(pair, tail)) => match *pair {
            FlatValue(Value::VPair(fst, snd)) => match (*fst, *snd) {
                (FlatValue(Value::VInt(fst)), FlatValue(Value::VInt(snd))) => {
                    acc.push((fst, snd));
                    vlist_of_pairs_to_vec_of_pairs(*tail, acc);
                }
                bad => error!(
                    "vlist_of_pairs_to_vec_of_pairs: impossible: bad types: {:?}",
                    bad
                ),
            },
            bad => error!(
                "vlist_of_pairs_to_vec_of_pairs: impossible: bad types: {:?}",
                bad
            ),
        },
        FlatValue(Value::VNil) => {}
        bad => error!(
            "vlist_of_pairs_to_vec_of_pairs: impossible: bad types: {:?}",
            bad
        ),
    }
}

fn score_meme(
    meme_eh: EntryHash,
    score_comp_ie_hh: HeaderHash,
) -> ExternResult<(HeaderHash, SensemakerEntry, HeaderHash, SensemakerEntry)> {
    let reaction_ie_links = get_links(meme_eh, Some(LinkTag::new(REACTION_TAG)))?;
    let mut reaction_ie_hh_s: Vec<HeaderHash> = vec![];
    for link in reaction_ie_links {
        match get_sensemaker_entry(link.target) {
            Ok((hh, ie)) => {
                check_schemes_unify(reaction_sc(), ie.output_scheme)?;
                reaction_ie_hh_s.push(hh);
            }
            Err(err) => {
                debug!("get_sensemaker_entry: err: {}", err);
            }
        }
    }

    let reaction_list_ie = pack_ies_into_list_ie(reaction_ie_hh_s)?;
    let reaction_list_ie_hh = create_entry(&reaction_list_ie)?;

    let input = CreateSensemakerEntryInputParse {
        expr: AGGREGATOR_FN_COMP.to_string(),
        args: vec![],
    };
    let (agg_comp_ie_hh, _agg_comp_ie) = create_sensemaker_entry_parse(input)?;
    let aggregated_ie = mk_application_ie(vec![agg_comp_ie_hh, reaction_list_ie_hh])?;
    let aggregated_reaction_list_ie_hh = create_entry(&aggregated_ie)?;
    let score_comp_application_ie = mk_application_ie(vec![
        score_comp_ie_hh,
        aggregated_reaction_list_ie_hh.clone(),
    ])?;

    let score_comp_application_ie_hh = create_entry(&score_comp_application_ie)?;

    Ok((
        score_comp_application_ie_hh,
        score_comp_application_ie,
        aggregated_reaction_list_ie_hh,
        aggregated_ie,
    ))
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
                create_sensemaker_entry_full(CreateSensemakerEntryInput { expr, args: vec![] })?;
            let _link_hh = create_link(rtmi.meme_eh, ie_eh, LinkTag::new(REACTION_TAG));
            Ok(true)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ScoreComputation {
    name: String,
    expr_str: String,
    nsc_eh: EntryHash,
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
    let mut score_comps: Vec<ScoreComputation> = vec![];
    for lnk in (get_links(
        hash_entry(NamedScoreComputationRoot)?,
        Some(LinkTag::new(NAMED_SCORE_COMP_TAG)),
    )?)
    .iter()
    {
        let res: ExternResult<ScoreComputation> = {
            let nsc_eh = lnk.target.clone();

            let (_nsc_hh, nsc) = get_nsc(nsc_eh.clone())?;
            let (_ie_eh, ie) = get_sensemaker_entry_by_headerhash(nsc.score_comp_ie_hh)?;

            let () = check_schemes_unify(score_comp_sc(), ie.output_scheme)?;

            let expr_str = format!("{:?}", ie.operator);
            Ok(ScoreComputation {
                name: nsc.name,
                expr_str,
                nsc_eh,
            })
        };
        match res {
            Ok(score_comp) => {
                score_comps.push(score_comp);
            }
            Err(err) => debug!("get_score_computations: error: {}", err),
        }
    }

    Ok(score_comps)
}

fn get_nsc(nsc_eh: EntryHash) -> ExternResult<(HeaderHash, NamedScoreComputation)> {
    let element = (match get(nsc_eh.clone(), GetOptions::content())? {
        Some(el) => Ok(el),
        None => Err(WasmError::Guest(format!(
            "could not dereference arg: {}",
            nsc_eh
        ))),
    })?;
    let nsc_hh = element.header_address().clone();
    let nsc: NamedScoreComputation = (match element.into_inner().1.to_app_option()? {
        Some(nsc) => Ok(nsc),
        None => Err(WasmError::Guest(format!("non-present arg: {}", nsc_eh))),
    })?;
    Ok((nsc_hh, nsc))
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

#[derive(Debug, Serialize, Deserialize)]
struct CreateScoreComputationInput {
    name: String,
    comp: String,
}

#[hdk_entry]
struct NamedScoreComputationRoot;

/// returns true if created, false if already exists
fn create_nsc_root_if_needed() -> ExternResult<bool> {
    match get(
        hash_entry(&NamedScoreComputationRoot)?,
        GetOptions::content(),
    )? {
        None => {
            let _hh = create_entry(&NamedScoreComputationRoot)?;
            Ok(true)
        }
        Some(_) => Ok(false),
    }
}

#[hdk_entry]
struct NamedScoreComputation {
    name: String,
    score_comp_se_hh: HeaderHash,
}

/// takes a string name, and a string which should parse to a `rep_lang` Expr
/// with type:
///   List (Int, Int) -> Int
/// and returns the `HeaderHash` of the created `NamedScoreComputation`, which
/// holds the name and the HeaderHash of the SensemakerEntry which houses the
/// score computation.
#[hdk_extern]
fn create_score_computation(csci: CreateScoreComputationInput) -> ExternResult<EntryHash> {
    debug!("{}", csci.comp);
    let input = CreateSensemakerEntryInputParse {
        expr: csci.comp,
        args: vec![],
    };
    let (ie_hh, ie) = create_sensemaker_entry_parse(input)?;

    // check IE scheme is right
    let () = check_schemes_unify(score_comp_sc(), ie.output_scheme)?;

    let nsc = NamedScoreComputation {
        name: csci.name,
        score_comp_ie_hh: ie_hh,
    };

    let nsc_eh = hash_entry(&nsc)?;
    match get(nsc_eh.clone(), GetOptions::content())? {
        // if nsc doesn't exist, create it and link it
        None => {
            let _nsc_hh = create_entry(&nsc)?;
            create_nsc_root_if_needed()?;
            create_link(
                hash_entry(NamedScoreComputationRoot)?,
                nsc_eh.clone(),
                LinkTag::new(NAMED_SCORE_COMP_TAG),
            )?;
        }
        Some(_) => {}
    };

    Ok(nsc_eh)
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
