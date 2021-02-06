use crate::reader::Sent;
use crate::utils::cartesian_product;
use crate::utils::{linspace, noparam, pack};
use clap::Clap;

mod align_hard;
mod align_soft;
mod optimizer;
mod utils;

use utils::{cli::Opts, evaluator, reader};

fn main() {
    let opts: Opts = Opts::parse();

    let (sents, (vocab1, vocab2)) = reader::load_all(opts.file1, opts.file2);

    let alignment_probs = match opts.soft.as_str() {
        "ibm1" => align_soft::ibm1::ibm1(&sents, &vocab1, &vocab2),
        "levenstein" => align_soft::misc::levenstein(&sents, &vocab1, &vocab2),
        _ => panic!("Unknown soft algorithm"),
    };

    let alignment = match opts.hard.as_str() {
        "argmax" => align_hard::a1_argmax(&alignment_probs),
        "basic" => {
            let algn_a1 = align_hard::a1_argmax(&alignment_probs);
            let algn_a2 = align_hard::a2_threshold(&alignment_probs, 0.01);
            optimizer::intersect_algn(Some(algn_a1), algn_a2).unwrap()
        }
        "search" => {
            const GOLD_COUNT: usize = 20;
            let algn_gold = if let Some(file) = opts.gold {
                reader::load_gold(file, GOLD_COUNT, false)
            } else {
                panic!("Gold alignments not supplied (only top N are required)")
            };

            let sents_rev = sents
                .iter()
                .map(|(x, y)| (y.clone(), x.clone()))
                .collect::<Vec<(Sent, Sent)>>();
            let alignment_probs_rev =
                &align_soft::ibm1::ibm1(&sents_rev, &vocab2, &vocab1)[..GOLD_COUNT];
            let alignment_probs_diagonal = align_soft::misc::diagonal(&sents[..GOLD_COUNT]);
            let alignment_probs_levenstein =
                align_soft::misc::levenstein(&sents[..GOLD_COUNT], &vocab1, &vocab2);
            let alignment_probs = &alignment_probs[..GOLD_COUNT];

            let (algn, _params, _aer) = optimizer::gridsearch(
                &[
                    //(pack(&noparam()), optimizer::AlgnMergeAction::INTERSECT, &|p: &[f32]| align_hard::a1_argmax(&alignment_probs),
                    (
                        pack(&linspace(0.0, 0.05, 2)),
                        optimizer::AlgnMergeAction::INTERSECT,
                        &|p: &[f32]| align_hard::a2_threshold(&alignment_probs, p[0]),
                    ),
                    (
                        pack(&linspace(0.0, 0.0, 1)),
                        optimizer::AlgnMergeAction::INTERSECT,
                        &|p: &[f32]| align_hard::a3_threshold_dynamic(&alignment_probs, p[0]),
                    ),
                    (
                        pack(&linspace(0.95, 1.0, 4)),
                        optimizer::AlgnMergeAction::INTERSECT,
                        &|p: &[f32]| align_hard::a4_threshold_dynamic(&alignment_probs, p[0]),
                    ),
                    (
                        pack(&linspace(0.4, 0.8, 5)),
                        optimizer::AlgnMergeAction::INTERSECT,
                        &|p: &[f32]| {
                            align_hard::a4_threshold_dynamic(&alignment_probs_diagonal, p[0])
                        },
                    ),
                    (
                        cartesian_product(vec![linspace(0.0, 0.2, 3), linspace(0.1, 0.3, 8)]),
                        optimizer::AlgnMergeAction::INTERSECT,
                        &|p: &[f32]| {
                            align_hard::a2_threshold(
                                &align_soft::misc::blur(&alignment_probs, p[0]),
                                p[1],
                            )
                        },
                    ),
                    (
                        pack(&linspace(0.95, 1.0, 4)),
                        optimizer::AlgnMergeAction::INTERSECT,
                        &|p: &[f32]| {
                            evaluator::alignment_reverse(&align_hard::a4_threshold_dynamic(
                                &alignment_probs_rev,
                                p[0],
                            ))
                        },
                    ),
                    (
                        pack(&linspace(0.7, 1.0, 4)),
                        optimizer::AlgnMergeAction::JOIN,
                        &|p: &[f32]| align_hard::a2_threshold(&alignment_probs_levenstein, p[0]),
                    ),
                ],
                &algn_gold,
            );
            algn
        }
        _ => panic!("Unknown hard algorithm"),
    };

    // print alignments
    for sent_align in alignment {
        println!(
            "{}",
            sent_align
                .iter()
                .map(|(pos1, pos2)| format!("{}-{}", pos1, pos2))
                .collect::<Vec<String>>()
                .join(" ")
        );
    }
}
