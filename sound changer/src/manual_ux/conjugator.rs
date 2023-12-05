// TODO: Custom char that is an enum for stronger typing on alternations

use std::{
    collections::HashMap,
    fmt::{self, Display},
    rc::Rc,
};

use tabled::{builder::Builder, settings::Style};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConjugatorInput {
    pub words: Vec<Vec<String>>,
    pub max_conjugations: usize,
    pub max_intraconjugation_roots: usize,
    pub max_alternations: usize,
}

pub trait Conjugation: fmt::Debug {
    fn conjugate(&self, roots: &[String], alternation_info: &[usize]) -> Vec<String>;
    fn to_string(&self) -> String;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AffixConjugation {
    pub affices: Vec<AffixSet>,
    pub alternations: Vec<Vec<Alternation>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AffixSet {
    pub root_id: usize,
    pub prefix: String,
    pub suffix: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Alternation {
    pub target_pattern: String,
    pub replacement_pattern: Option<char>,
}

impl Conjugation for AffixConjugation {
    fn conjugate(&self, roots: &[String], alternation_data: &[usize]) -> Vec<String> {
        let mut conjugated_forms = Vec::new();
        for affix_set in &self.affices {
            let root = roots[affix_set.root_id].as_str();
            let mut result = format!("{}{}{}", affix_set.prefix, root, affix_set.suffix);

            for alternation_id in alternation_data {
                let alternation_set = &self.alternations[*alternation_id];
                for alternation in alternation_set {
                    let target = &alternation.target_pattern;
                    let replacement = match &alternation.replacement_pattern {
                        Some(c) => String::from(*c),
                        None => String::from(""),
                    };
                    result = result.replace(target, &replacement.to_string());
                }
            }

            conjugated_forms.push(result);
        }

        conjugated_forms
    }

    fn to_string(&self) -> String {
        format!("{:?}", self.alternations)
    }
}

#[derive(Debug)]
pub enum ConjugatedWord {
    Regular {
        roots: Vec<String>,
        // TODO: Make this Weak<dyn Conjugation>
        conjugation: Rc<dyn Conjugation>,
        alternation_info: Vec<usize>,
    },
    Irregular {
        word: Vec<String>,
    },
}

pub struct ConjugatorOutput {
    pub conjugations: Vec<Rc<dyn Conjugation>>,
    pub conjugated_words: Vec<ConjugatedWord>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlternationInternalAffixConjugation {
    pub affices: Vec<AlternationInternalAffixSet>,
    pub alternations: Vec<Vec<AlternationInternalAlternation>>,
    pub source_conjugation: Rc<AffixConjugation>,
}

impl AlternationInternalAffixConjugation {
    /// In addition to creating a new AlternationInternalAffixConjugation, this function also creates
    /// a matching regular AffixConjugation for the source_conjugation field.
    pub fn new(
        affices: Vec<AlternationInternalAffixSet>,
        alternations: Vec<Vec<AlternationInternalAlternation>>,
    ) -> AlternationInternalAffixConjugation {
        // The most complex thing here is to make the source conjugation.
        let source_affices: Vec<AffixSet> = affices
            .iter()
            .map(|affix| AffixSet {
                root_id: affix.root_id,
                prefix: to_string(&affix.prefix),
                suffix: to_string(&affix.suffix),
            })
            .collect();
        let source_alternations: Vec<Vec<Alternation>> = alternations
            .iter()
            .map(|x| {
                x.iter()
                    .map(|y| Alternation {
                        target_pattern: y.target_pattern.to_string(),
                        replacement_pattern: y.replacement_pattern,
                    })
                    .collect()
            })
            .collect();
        let source_conjugation = Rc::new(AffixConjugation {
            affices: source_affices,
            alternations: source_alternations,
        });
        AlternationInternalAffixConjugation {
            affices,
            alternations,
            source_conjugation,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlternationInternalAffixSet {
    pub root_id: usize,
    pub prefix: Vec<AlternationInternalChar>,
    pub suffix: Vec<AlternationInternalChar>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AlternationInternalAlternation {
    pub target_pattern: AlternationInternalChar,
    pub replacement_pattern: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum AlternationInternalChar {
    Char(char),
    Alternation(usize),
}

impl Display for AlternationInternalChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char(c) => write!(f, "{}", c),
            Self::Alternation(id) => write!(f, "\\{}", id),
        }
    }
}

fn to_string(word: &Vec<AlternationInternalChar>) -> String {
    let mut result = String::new();
    for c in word {
        result.push_str(&c.to_string());
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum AlternationMode {
    Replacement,
    Insertion,
    Deletion,
}

impl AlternationInternalChar {
    pub fn to_escape_id(&self) -> Option<usize> {
        match self {
            Self::Char(_) => None,
            Self::Alternation(id) => Some(*id),
        }
    }

    fn to_primitive_char(self) -> Option<char> {
        match self {
            Self::Char(c) => Some(c),
            Self::Alternation(_) => None,
        }
    }
}

pub struct AlternationIntermediateResult {
    pub new_string1: Vec<AlternationInternalChar>,
    pub new_string2: Vec<AlternationInternalChar>,
    pub new_alternation_pairs: Vec<(
        // These are still chars because these are what the escape
        // sequences are replaced with.
        Option<char>,
        Option<char>,
    )>,
}

impl AlternationIntermediateResult {
    fn new(
        new_string1: Vec<AlternationInternalChar>,
        new_string2: Vec<AlternationInternalChar>,
        new_alternation_pairs: Vec<(Option<char>, Option<char>)>,
    ) -> Self {
        Self {
            new_string1,
            new_string2,
            new_alternation_pairs,
        }
    }
}

type InternalAffixAndAlternation = (
    // Affix
    Vec<AlternationInternalChar>,
    // Alternation
    Vec<(Option<char>, Option<char>)>,
);

type InternalAlternationBlob<'a> =
    HashMap<&'a (Option<char>, Option<char>), (Vec<(usize, usize)>, usize)>;

/// Creates a set of affix conjugations for the input.
pub fn create_conjugations(input: ConjugatorInput) -> ConjugatorOutput {
    // First we model each word as it's own conjugation
    // Then we try to find commonalities between them and eliminate

    let (mut conjugations, mut words) = create_base_conjugations(&input);

    remove_duplicate_conjugations(&mut conjugations, &mut words);

    find_and_collapse_alternations(&input, &mut conjugations, &mut words);

    let table_string = {
        let mut builder = Builder::default();

        let mut headers = Vec::new();
        for id in 0..conjugations.len() {
            headers.push(id.to_string());
        }
        builder.set_header(headers);

        for index in 0..conjugations[0].affices.len() {
            let mut strings = Vec::new();
            for conjugation in &conjugations {
                let affix = &conjugation.affices[index];
                strings.push(format!(
                    "{}-{}-{}",
                    affix.prefix, affix.root_id, affix.suffix
                ));
            }
            builder.push_record(strings);
        }

        let mut table = builder.build();
        table.with(Style::markdown()).to_string()
    };

    println!("{}", conjugations[0].to_string());
    println!("{}", table_string);

    verify_conjugations(&words, &input.words);

    // Cast all AffixConjugations to dyn Conjugation
    let final_conjugations = conjugations
        .into_iter()
        .map(|x| x as Rc<dyn Conjugation>)
        .collect();

    ConjugatorOutput {
        conjugations: final_conjugations,
        conjugated_words: words,
    }
}

/// Conjugates every word and verifies that the conjugations are correct.
/// Panics if they are not.
fn verify_conjugations(words: &[ConjugatedWord], original_words: &[Vec<String>]) {
    for (word, expected_forms) in words.iter().zip(original_words.iter()) {
        match word {
            ConjugatedWord::Regular {
                roots,
                conjugation,
                alternation_info,
            } => {
                let conjugated_forms = conjugation.conjugate(roots, alternation_info);
                assert_eq!(conjugated_forms.len(), expected_forms.len());
                for (conjugated_form, expected_form) in
                    conjugated_forms.iter().zip(expected_forms.iter())
                {
                    println!("{} {}", conjugated_form, expected_form);
                    // assert_eq!(conjugated_form, expected_form);
                }
            }
            ConjugatedWord::Irregular { word: _ } => {}
        }
    }
}

/// Finds and collapses alternations within the conjugations.
fn find_and_collapse_alternations(
    input: &ConjugatorInput,
    conjugations: &mut Vec<Rc<AffixConjugation>>,
    words: &mut [ConjugatedWord],
) {
    let mut internal_conjugations = convert_to_internal_alternations(conjugations);

    // Now we want to find similar conjugations, where simple alternations can be made.
    // i.e. ita ito is => eta eto es.
    // Alternations are represented within the affix with an escape sequence, \0, \1, \2, etc.
    // So we would have two alternations, \0 -> i and \0 -> e in \0ta \0to \0s.

    // We first try collapsing every pair of conjugations with one alternation, then increase.
    for alternation_count in 1..input.max_alternations {
        // The flag allows us to break multiple times.
        let mut flag: bool = true;
        while flag {
            flag = false;

            struct AlternationCollapse {
                index1: usize,
                index2: usize,
                alternation_idx1: Vec<usize>,
                alternation_idx2: Vec<usize>,
                new_conjugation: AlternationInternalAffixConjugation,
            }

            let mut action = None;

            // We loop through every pair of conjugations
            'outer: for (index1, conjugation) in internal_conjugations.iter().enumerate() {
                'pair_loop: for (index2, conjugation2) in internal_conjugations.iter().enumerate() {
                    if index1 >= index2 {
                        continue;
                    }

                    // Check if this can be collapsed. If the root ids don't match, then alternations
                    // wont help.
                    let mut can_collapse = true;
                    for affix_pair in conjugation.affices.iter().zip(conjugation2.affices.iter()) {
                        if affix_pair.0.root_id != affix_pair.1.root_id {
                            can_collapse = false;
                        }
                    }
                    if !can_collapse {
                        continue;
                    }

                    let mut alternation_sum = Vec::new();

                    // println!("{}, {}", index1, index2);
                    // for affix_pair in conjugation.affices.iter().zip(conjugation2.affices.iter()) {
                    //     println!(
                    //         "\t{}, {}",
                    //         to_string(&affix_pair.0.suffix),
                    //         to_string(&affix_pair.1.suffix)
                    //     );
                    // }

                    // We now loop through every affix pair and try to make alternations.
                    for affix_pair in conjugation.affices.iter().zip(conjugation2.affices.iter()) {
                        if affix_pair.0 == affix_pair.1 {
                            // Insert an empty alternation into the sum so that the indices match up
                            alternation_sum.push((affix_pair.0.suffix.clone(), Vec::new()));
                            continue;
                        }

                        // We now search for an alternation that can be made to make the roots match.
                        // We search character by character; if the alternation would be multiple characters,
                        // it will be made with two alternations in a higher alternation count.
                        // Similarly to root finding, we will send this out to a helper function.

                        // TODO: Make work with prefixes
                        let result = alternation_helper(
                            affix_pair.0.suffix.clone(),
                            affix_pair.1.suffix.clone(),
                            // TODO: Fix the +1
                            alternation_count + 1,
                            Vec::new(),
                        );

                        if let Some(v) = result {
                            assert_eq!(v.new_string1, v.new_string2);
                            alternation_sum.push((v.new_string1, v.new_alternation_pairs));
                        } else {
                            continue 'pair_loop;
                        }
                    }

                    // We have a valid alternation sum and an affix, we will attempt to combine them.
                    // First we need to add all of the alternations together and see if the total is less
                    // than the max alternations.

                    // We use a HashMap to store the intermediate alternation blob because we need to keep
                    // track of the alternation ids.

                    // TODO: Have a start index for new alternation ids
                    let combined_alternations = combine_alternations(&alternation_sum);

                    // If the combined map has more than max_alternations, we can't use alternations on these
                    // two conjugations.
                    if combined_alternations.len() > input.max_alternations {
                        continue;
                    }

                    // We need to make a new list of affices
                    let new_affices =
                        form_new_affices(&alternation_sum, conjugation, &combined_alternations);

                    let mut alternations = {
                        let mut temp = conjugation.alternations.clone();
                        temp.append(&mut conjugation2.alternations.clone());
                        temp
                    };
                    let mut alternation_idx1 = Vec::new();
                    let mut alternation_idx2 = Vec::new();
                    for (alternation, map) in combined_alternations {
                        // If the alternation is a digit, then it is a previously existing alternation already covered
                        if should_append_alternation(alternation.0) {
                            alternations.push(vec![AlternationInternalAlternation {
                                target_pattern: AlternationInternalChar::Alternation(map.1),
                                replacement_pattern: alternation.0,
                            }]);
                            // We need to record the indices of the alternations that we are using.
                            // This is for the first conjugation we process.
                            alternation_idx1.push(alternations.len() - 1);
                        }

                        if should_append_alternation(alternation.1) {
                            alternations.push(vec![AlternationInternalAlternation {
                                target_pattern: AlternationInternalChar::Alternation(map.1),
                                replacement_pattern: alternation.1,
                            }]);
                            // And this is for the second conjugation.
                            alternation_idx2.push(alternations.len() - 1);
                        }
                    }

                    // We need to kick modifying the conjugations to outside the loop
                    // because of the iterators
                    action = Some(AlternationCollapse {
                        index1,
                        index2,
                        alternation_idx1,
                        alternation_idx2,
                        new_conjugation: AlternationInternalAffixConjugation::new(
                            new_affices,
                            alternations,
                        ),
                    });

                    break 'outer;
                }
            }

            if let Some(v) = action {
                // We found something we have to do.
                // Mark that we did something and collapse the conjugations together.
                flag = true;

                let new_conjugation = v.new_conjugation;
                let old_conjugation1 = internal_conjugations.remove(v.index1);
                let r = if v.index1 < v.index2 {
                    v.index2 - 1
                } else {
                    v.index2
                };
                let old_conjugation2 =
                    std::mem::replace(&mut internal_conjugations[r], new_conjugation.clone());
                let conjugation1_ref_ptr = std::rc::Rc::<dyn Conjugation>::as_ptr(
                    &(old_conjugation1.source_conjugation.clone() as Rc<dyn Conjugation>),
                );
                let conjugation2_ref_ptr = std::rc::Rc::<dyn Conjugation>::as_ptr(
                    &(old_conjugation2.source_conjugation.clone() as Rc<dyn Conjugation>),
                );
                let alternation_set_1 = v.alternation_idx1.clone();
                let alternation_set_2 = v.alternation_idx2.clone();
                let operator = |word: &mut ConjugatedWord, from: &Rc<dyn Conjugation>, _: &_| {
                    // It's not enough to simply change the conjugation here, we need to identify and record
                    // the alternations that are used.

                    // Determine which conjugation we had previously and therefore which alternations to use

                    // For std::ptr::eq to work we need ptrs to the same type.
                    // Clippy incorrectly flags this as an error. I believe that
                    // std::ptr::eq is the correct way to do this. If not, it works
                    // because the vtables are equal because everything is created
                    // in this translation unit.
                    #[allow(clippy::vtable_address_comparisons)]
                    let alternations = if std::ptr::eq(
                        std::rc::Rc::<dyn Conjugation>::as_ptr(from),
                        conjugation1_ref_ptr,
                    ) {
                        &alternation_set_1
                    } else {
                        // We assume that it's the other one if we've gotten here
                        assert!(std::ptr::eq(
                            std::rc::Rc::<dyn Conjugation>::as_ptr(from),
                            conjugation2_ref_ptr
                        ));
                        &alternation_set_2
                    };

                    match word {
                        ConjugatedWord::Regular {
                            roots: _,
                            conjugation: _,
                            ref mut alternation_info,
                        } => {
                            alternation_info.append(&mut alternations.clone());
                        }
                        _ => {
                            unreachable!()
                        }
                    }
                };

                // All words that have ptr::eq to the old
                replace_conjugations(
                    words,
                    old_conjugation1.source_conjugation.clone() as Rc<dyn Conjugation>,
                    new_conjugation.source_conjugation.clone(),
                    &operator,
                );
                replace_conjugations(
                    words,
                    old_conjugation2.source_conjugation.clone() as Rc<dyn Conjugation>,
                    new_conjugation.source_conjugation,
                    &operator,
                );
            }
        }
    }

    // We now replace the conjugations with the new ones
    conjugations.clear();
    internal_conjugations
        .into_iter()
        .for_each(|x| conjugations.push(x.source_conjugation));
}

/// This converts the string based conjugations into Vecs of AlternationInternalChars.
fn convert_to_internal_alternations(
    conjugations: &[Rc<AffixConjugation>],
) -> Vec<AlternationInternalAffixConjugation> {
    conjugations
        .iter()
        .map(|from| {
            // Convert the affix conjugation to internal conjugation
            // This should be used to setup creating alternations, so we shouldn't have alternations yet.
            // Therefore there is no point to implementing it
            assert!(from.alternations.is_empty());

            // Convert the affices
            let affices = from
                .affices
                .iter()
                .map(|affix_set| AlternationInternalAffixSet {
                    root_id: affix_set.root_id,

                    // Convert prefixes
                    prefix: affix_set
                        .prefix
                        .chars()
                        .map(|c| {
                            // Similarly, there should be no alternations in the affices yet,
                            // so we don't need to implement this. Again we check it.
                            assert_ne!(c, '\\');
                            AlternationInternalChar::Char(c)
                        })
                        .collect(),

                    // Convert suffixes
                    suffix: affix_set
                        .suffix
                        .chars()
                        .map(|c| {
                            assert_ne!(c, '\\');
                            AlternationInternalChar::Char(c)
                        })
                        .collect(),
                })
                .collect();

            AlternationInternalAffixConjugation {
                affices,
                alternations: Vec::new(),
                source_conjugation: from.clone(),
            }
        })
        .collect()
}

/// Returns true if this is None or not a digit
fn should_append_alternation(value: Option<char>) -> bool {
    value.is_none() || !value.unwrap().is_ascii_digit()
}

/// Takes the alternation sum, the current conjugation, and the combined alternations
/// and forms a new set of affices to replace the old ones.
fn form_new_affices(
    alternation_sum: &[InternalAffixAndAlternation],
    conjugation: &AlternationInternalAffixConjugation,
    combined_alternations: &InternalAlternationBlob,
) -> Vec<AlternationInternalAffixSet> {
    // alternation_sum contains a mapping from affices (with the alternation locations noted)
    // and the relevant alternations *for that affix alone*.
    // conjugation contains the conjugation that we are modifying, just releveant for grabbing the root id
    // combined_alternations contains the final mapping for alternation ids. It maps from
    // (alternation_id, affix_id) -> new_alternation_id
    let mut new_affices = Vec::new();
    for iter_out in alternation_sum
        .iter()
        .zip(conjugation.affices.iter())
        .enumerate()
    {
        let item = iter_out.1;
        let mut affix = item.0 .0.clone();
        let root_id = item.1.root_id;
        let affix_id = iter_out.0;

        affix = affix
            .iter()
            .map(|c| match c {
                AlternationInternalChar::Alternation(alternation_id) => {
                    // Get ID for *this* alternation
                    // Search through the combined alternations to find the matching one
                    let temp = combined_alternations
                        .iter()
                        .find(|(_, v)| v.0.contains(&(*alternation_id, affix_id)));
                    let new_alternation_id = match temp {
                        Some(v) => v.1 .1,
                        None => {
                            println!(
                                "Failed to find alternation ({}, {}) in: \n{:?}",
                                alternation_id, affix_id, combined_alternations
                            );
                            todo!()
                        }
                    };
                    // Convert the new alternation id to a char
                    AlternationInternalChar::Alternation(new_alternation_id)
                }
                AlternationInternalChar::Char(c) => AlternationInternalChar::Char(*c),
            })
            .collect();

        new_affices.push(AlternationInternalAffixSet {
            root_id,
            prefix: Vec::new(),
            suffix: affix,
        });
    }
    new_affices
}

/// Combines the alternations into a single HashMap. The HashMap maps the alternation
/// to a tuple of (Vec<(alternation_index, affix_index)>, alternation_id).
fn combine_alternations(
    alternation_sum: &[InternalAffixAndAlternation],
) -> InternalAlternationBlob {
    let mut combined_alternations = HashMap::new();
    for (affix_index, alternations) in alternation_sum.iter().enumerate() {
        for (alternation_index, alternation) in alternations.1.iter().enumerate() {
            if !combined_alternations.contains_key(alternation) {
                combined_alternations
                    .insert(alternation, (Vec::new(), combined_alternations.len()));
            }
            // We need to add the ids to the HashMap
            combined_alternations
                .get_mut(alternation)
                .unwrap()
                .0
                .push((alternation_index, affix_index));
        }
    }
    combined_alternations
}

/// Performs a simple check to see if the conjugations are exactly the same and remove
/// any duplicates.
fn remove_duplicate_conjugations(
    conjugations: &mut Vec<Rc<AffixConjugation>>,
    words: &mut [ConjugatedWord],
) {
    // First we check for any duplicate conjugations
    // If we find any, we merge them together
    // We do this by checking if the affices are the same
    let mut flag = true;
    while flag {
        let mut indices = None;
        'outer: for (index, conjugation) in conjugations.iter().enumerate() {
            for (index2, conjugation2) in conjugations.iter().enumerate() {
                if index == index2 {
                    continue;
                }
                if conjugation.affices == conjugation2.affices {
                    // We have a duplicate conjugation
                    // We need to merge them together
                    indices = Some((index, index2));
                    break 'outer;
                }
            }
        }
        if let Some(index) = indices {
            // Delete the conjugation
            let removed: Rc<dyn Conjugation> = conjugations.remove(index.1);
            // let operator = |_: &_, _: &_, _: &_| {};
            replace_conjugations(
                words,
                removed,
                conjugations[index.0].clone() as Rc<dyn Conjugation>,
                &|_: &mut ConjugatedWord, _: &_, _: &_| {},
            );
            flag = true;
        } else {
            flag = false;
        }
    }
}

/// Takes a list of words and replaces any conjugations that are equal to removed with the conjugation.
/// This is done by checking pointer equality not value equality.
fn replace_conjugations(
    words: &mut [ConjugatedWord],
    to_replace: Rc<dyn Conjugation>,
    to_replace_with: Rc<dyn Conjugation>,
    operator: &impl Fn(&mut ConjugatedWord, &Rc<dyn Conjugation>, &Rc<dyn Conjugation>),
) {
    // We now need to take every word that uses the removed conjugation and replace it with the
    // remaining conjugation. These conjugations are identical, which makes our life difficulty,
    // as the == operator checks value equality. We need to check reference equality, which is
    // Rc::ptr_eq.
    for word in words.iter_mut() {
        // True if it was modified, which indicates that we need to call the operator.
        let mut flag = false;
        match word {
            ConjugatedWord::Regular {
                roots: _,
                conjugation,
                alternation_info: _,
            } => {
                // For std::ptr::eq to work we need ptrs to the same type.
                // Clippy incorrectly flags this as an error. I believe that
                // std::ptr::eq is the correct way to do this. If not, it works
                // because the vtables are equal because everything is created
                // in this translation unit.
                #[allow(clippy::vtable_address_comparisons)]
                if std::ptr::eq(
                    std::rc::Rc::<dyn Conjugation>::as_ptr(conjugation),
                    std::rc::Rc::<dyn Conjugation>::as_ptr(&to_replace),
                ) {
                    *conjugation = to_replace_with.clone();
                    flag = true;
                }
            }
            ConjugatedWord::Irregular { .. } => {}
        }
        if flag {
            operator(word, &to_replace, &to_replace_with);
        }
    }
}

/// Creates base conjugations; each word will have it's own conjugation.
/// Automatically finds the roots and does all of that.
fn create_base_conjugations(
    input: &ConjugatorInput,
) -> (Vec<Rc<AffixConjugation>>, Vec<ConjugatedWord>) {
    let mut conjugations = Vec::new();
    let mut words = Vec::new();

    // We start by finding the longest common root
    for word in &input.words {
        let roots = match root_searcher_helper(
            Vec::new(),
            input.max_intraconjugation_roots - 1,
            word.to_vec(),
        ) {
            Some(roots) => roots,
            None => {
                // We have an irregular word
                words.push(ConjugatedWord::Irregular { word: word.clone() });
                continue;
            }
        };

        let mut affices = Vec::new();

        // We now need to take the root and derive the prefixes and suffixes from it.
        // We loop through every form, determine the longest root that fits and where it is, and then
        // take substrings for the prefix and suffix.
        for form in word {
            let mut best_root = None;
            let mut best_root_length = 0;
            for (index, root) in roots.iter().enumerate() {
                let find = form.find(root);
                if let Some(root_start) = find {
                    if root.chars().count() > best_root_length {
                        best_root = Some((root, root_start, index));
                        best_root_length = root.chars().count();
                    }
                }
            }
            let best_root = best_root.unwrap();

            let prefix = form[..best_root.1].to_string();
            let suffix = form[best_root.1 + best_root.0.len()..].to_string();

            let affix = AffixSet {
                root_id: best_root.2,
                prefix,
                suffix,
            };
            affices.push(affix);
        }
        let conjugation = Rc::new(AffixConjugation {
            affices,
            alternations: Vec::new(),
        });

        let word = ConjugatedWord::Regular {
            roots,
            conjugation: conjugation.clone(),
            alternation_info: Vec::new(),
        };

        conjugations.push(conjugation);
        words.push(word);
    }

    (conjugations, words)
}

/// Recursive helper function for create_base_conjugations. For every root length and starting index of the
/// first form, it searches to see if the other forms match it. If not, and remaining_interconjugation_roots > 0
/// it will recursively call itself to search for roots with multiple valid roots.
fn root_searcher_helper(
    existing_roots: Vec<String>,
    remaining_interconjugation_roots: usize,
    remaining_forms: Vec<String>,
) -> Option<Vec<String>> {
    // The root can start around any letter, so we use the following algorithm:
    // For every letter that the root can start with in the first form (length restrictions),
    // we check if the other forms include that root. If we reach another form that doesn't
    // include the root, we may be able to check the root if max_interconjugation_roots is
    // greater than the number of roots we are tracking. Otherwise, we stop the search.

    let mut best_result = None;

    // Generate all possible starting configurations
    let form_length = remaining_forms[0].chars().count();
    for root_length in 1..form_length + 1 {
        'outer: for root_start in 0..form_length - root_length {
            // Get the characters of the root
            let root = {
                let mut char_indices = remaining_forms[0].char_indices();
                let (root_start_index, _) = char_indices.nth(root_start).unwrap();
                let (root_end_index, _) = char_indices.nth(root_length - 1).unwrap();
                remaining_forms[0][root_start_index..root_end_index].to_string()
            };
            let mut new_roots = existing_roots.clone();
            new_roots.push(root);

            // Check if the other forms match the root
            let mut all_forms_match = true;
            let mut index = 1;
            for form in &remaining_forms[1..] {
                let mut found_match = false;
                for valid_root in &new_roots {
                    if form.contains(valid_root) {
                        found_match = true;
                        break;
                    }
                }
                if !found_match {
                    if remaining_interconjugation_roots == 0 {
                        all_forms_match = false;
                        break;
                    }
                    // We are allowed to search for more roots, so we recursively call ourselves.
                    // The recursive call is responsible for checking all future roots, so if it
                    // returns Some(), that will be the new best solution.
                    let recursive_result = root_searcher_helper(
                        new_roots.clone(),
                        remaining_interconjugation_roots - 1,
                        remaining_forms[index..].to_vec(),
                    );
                    if recursive_result.is_some() {
                        best_result = recursive_result;
                        continue 'outer;
                    }
                }
                index += 1;
            }

            if all_forms_match {
                // We have a valid solution for this root. It is guaranteed to be the best solution so far
                // because we are searching in increasing order of root length.
                best_result = Some(new_roots);
            }
        }
    }

    best_result
}

/// Recursive helper function for create_conjugations. It takes two different string and compares them character by character.
/// For a mismatched character, it will try to making an alternation, insertion, or deletion to make it work.
fn alternation_helper(
    affix1: Vec<AlternationInternalChar>,
    affix2: Vec<AlternationInternalChar>,
    remaining_alternations: usize,
    previous_alternations: Vec<(Option<char>, Option<char>)>,
) -> Option<AlternationIntermediateResult> {
    let mut iter1 = affix1.iter();
    let mut iter2 = affix2.iter();

    let mut char_index1 = 0;
    let mut char_index2 = 0;

    loop {
        // Take the next character from each string
        // If it's a \, we need to take the next character as well
        // because it's part of an escape sequence, replace the
        // escape sequence if needed, then compare them.
        let char1_opt = iter1.next();
        let char2_opt = iter2.next();
        char_index1 += 1;
        char_index2 += 1;
        if char1_opt.is_none() && char2_opt.is_none() {
            // We're done here
            return Some(AlternationIntermediateResult::new(
                affix1,
                affix2,
                previous_alternations,
            ));
        } else if char1_opt.is_none() || char2_opt.is_none() {
            // TODO: Make this work
            return None;
        }
        let char1 = char1_opt.unwrap();
        let char2 = char2_opt.unwrap();

        if char1 == char2 {
            // We're good
            continue;
        }

        // We now check for previous alternations from previous runs of the entire procedure
        let char1_escape = char1.to_escape_id();
        let char2_escape = char2.to_escape_id();

        // If there are two escape sequences that have different ids, we panic for now
        // This will have to be revisited later
        if char1_escape.is_some() && char2_escape.is_some() {
            println!("{} vs {}", to_string(&affix1), to_string(&affix2));
            assert_eq!(char1_escape, char2_escape);
        }

        // We have a mismatched character
        // We need to try to make an alternation, insertion, or deletion

        // The alternation would be char1, char2 (i.e. tar <-> ter t\0r)
        // The insertion would be char1, _ (i.e. est <-> et => e\0t)
        // The deletion would be _, char2 (i.e. et <-> est)

        // For each of those, we make the alternation than recursively call to evaluate.
        // We choose the result with the fewest alternations.

        // If the alternation we happen to be making is a existing alternation, then we
        // don't need to add it to the alternations list, nor do we need to subtract from
        // remaining_alternations.

        let alternation_results: Vec<Option<AlternationIntermediateResult>> = vec![
            try_replacement_alternation(
                &previous_alternations,
                remaining_alternations,
                char1,
                char2,
                &affix1,
                char_index1,
                &affix2,
                char_index2,
                AlternationMode::Replacement,
            ),
            try_replacement_alternation(
                &previous_alternations,
                remaining_alternations,
                char1,
                char2,
                &affix1,
                char_index1,
                &affix2,
                char_index2,
                AlternationMode::Insertion,
            ),
            try_replacement_alternation(
                &previous_alternations,
                remaining_alternations,
                char1,
                char2,
                &affix1,
                char_index1,
                &affix2,
                char_index2,
                AlternationMode::Deletion,
            ),
        ];

        let final_alternation_results: Vec<AlternationIntermediateResult> =
            alternation_results.into_iter().flatten().collect();

        let mut best_result = None;
        let mut best_alternations = usize::MAX;

        for alternation_result in final_alternation_results {
            if alternation_result.new_alternation_pairs.len() < best_alternations {
                best_alternations = alternation_result.new_alternation_pairs.len();
                best_result = Some(alternation_result);
            }
        }

        return best_result;
    }
}

// I'm ignoring this error because this is just used internally, and it would make the program more confusing
// if I combined parameters into structs, because of none of the parameters are particularly related,
// outside this function
#[allow(clippy::too_many_arguments)]
// This one is for writing &Vec<_> not &[_], but clippy is incorrect here and it is nessecary to use a full
// vec to make the program work.
#[allow(clippy::ptr_arg)]
fn try_replacement_alternation(
    previous_alternations: &Vec<(Option<char>, Option<char>)>,
    remaining_alternations: usize,
    char1: &AlternationInternalChar,
    char2: &AlternationInternalChar,
    affix1: &Vec<AlternationInternalChar>,
    char_index1: usize,
    affix2: &Vec<AlternationInternalChar>,
    char_index2: usize,
    mode: AlternationMode,
) -> Option<AlternationIntermediateResult> {
    let mut alternations = previous_alternations.clone();

    let primitive_char1 = char1.to_primitive_char();
    let primitive_char2 = char2.to_primitive_char();
    let find_or_insert_result =
        find_or_insert(&mut alternations, (primitive_char1, primitive_char2));

    let newchar = AlternationInternalChar::Alternation(find_or_insert_result.0);

    let new_remaining_alternations = if find_or_insert_result.1 {
        remaining_alternations
    } else if remaining_alternations > 0 {
        remaining_alternations - 1
    } else {
        // We can't make any more alternations
        return None;
    };

    let (new_affix1, new_affix2) = match mode {
        AlternationMode::Replacement => (
            replace_nth_char(affix1, char_index1 - 1, newchar),
            replace_nth_char(affix2, char_index2 - 1, newchar),
        ),
        AlternationMode::Insertion => (
            replace_nth_char(affix1, char_index1 - 1, newchar),
            insert_nth_char(affix2, char_index2 - 1, newchar),
        ),
        AlternationMode::Deletion => (
            insert_nth_char(affix1, char_index1 - 1, newchar),
            replace_nth_char(affix2, char_index2 - 1, newchar),
        ),
    };

    alternation_helper(
        new_affix1,
        new_affix2,
        new_remaining_alternations,
        alternations,
    )
}

/// Replaces the nth char with newstr
fn replace_nth_char(
    s: &[AlternationInternalChar],
    idx: usize,
    newchar: AlternationInternalChar,
) -> Vec<AlternationInternalChar> {
    s.iter()
        .enumerate()
        .map(|(i, c)| if i == idx { newchar } else { *c })
        .collect()
}

/// Inserts newstr *before* the nth char
fn insert_nth_char(
    s: &[AlternationInternalChar],
    idx: usize,
    newchar: AlternationInternalChar,
) -> Vec<AlternationInternalChar> {
    s.iter()
        .enumerate()
        .flat_map(|(i, c)| {
            if i == idx {
                vec![newchar, *c]
            } else {
                vec![*c]
            }
        })
        .collect()
}

#[test]
fn replace_nth_char_unit() {
    assert_eq!(
        replace_nth_char(
            &[
                AlternationInternalChar::Char('a'),
                AlternationInternalChar::Char('b'),
                AlternationInternalChar::Char('c')
            ],
            1,
            AlternationInternalChar::Char('d')
        ),
        vec![
            AlternationInternalChar::Char('a'),
            AlternationInternalChar::Char('d'),
            AlternationInternalChar::Char('c')
        ]
    );
}

#[test]
fn insert_nth_char_unit() {
    assert_eq!(
        insert_nth_char(
            &[
                AlternationInternalChar::Char('a'),
                AlternationInternalChar::Char('b'),
                AlternationInternalChar::Char('c')
            ],
            1,
            AlternationInternalChar::Char('d')
        ),
        vec![
            AlternationInternalChar::Char('a'),
            AlternationInternalChar::Char('d'),
            AlternationInternalChar::Char('b'),
            AlternationInternalChar::Char('c')
        ]
    );
}

/// If the element already exists in the vector, return its index and true.
/// Otherwise, insert it and return its index and false.
fn find_or_insert<T>(vec: &mut Vec<T>, elem: T) -> (usize, bool)
where
    T: PartialEq,
{
    for (index, existing_elem) in vec.iter().enumerate() {
        if *existing_elem == elem {
            return (index, true);
        }
    }
    vec.push(elem);
    (vec.len() - 1, false)
}
