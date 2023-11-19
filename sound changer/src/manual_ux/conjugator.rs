use std::{collections::HashMap, rc::Rc};

use tabled::{builder::Builder, settings::Style};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ConjugatorInput {
    pub words: Vec<Vec<String>>,
    pub max_conjugations: usize,
    pub max_interconjugation_roots: usize,
    pub max_alternations: usize,
}

pub trait Conjugation {
    fn conjugate(&self, roots: Vec<String>, alternation_data: usize) -> String;
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
    fn conjugate(&self, _roots: Vec<String>, _alternation_data: usize) -> String {
        todo!()
    }
}

pub enum ConjugatedWord {
    Regular {
        roots: Vec<String>,
        conjugation: Rc<dyn Conjugation>,
        alternation_id: usize,
    },
    Irregular {
        word: Vec<String>,
    },
}

pub struct ConjugatorOutput {
    pub conjugations: Vec<Rc<dyn Conjugation>>,
    pub conjugated_words: Vec<ConjugatedWord>,
}

type AlternationIntermediateResult = Option<(String, String, Vec<(Option<char>, Option<char>)>)>;

pub fn create_conjugations(input: ConjugatorInput) -> ConjugatorOutput {
    // First we model each word as it's own conjugation
    // Then we try to find commonalities between them

    let mut conjugations = create_base_conjugations(&input);

    remove_duplicate_conjugations(&mut conjugations);

    // Now we want to find similar conjugations, where simple alternations can be made.
    // i.e. ita ito is => eta eto es.
    // Alternations are represented within the affix with an escape sequence, \0, \1, \2, etc.
    // So we would have two alternations, \0 -> i and \0 -> e in \0ta \0to \0s.

    // We first try collapsing every pair of conjugations with one alternation, then increase.
    for alternation_count in 1..input.max_alternations {
        // The flag allows us to break multiple times.
        let mut flag = true;
        while flag {
            flag = false;

            let mut action = None;

            // We loop through every pair of conjugations
            'outer: for (index, conjugation) in conjugations.iter().enumerate() {
                'pair_loop: for (index2, conjugation2) in conjugations.iter().enumerate() {
                    if index >= index2 {
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
                            &affix_pair.0.suffix,
                            &affix_pair.1.suffix,
                            // TODO: Fix the +1
                            alternation_count + 1,
                            Vec::new(),
                        );

                        if let Some(v) = result {
                            assert_eq!(v.0, v.1);
                            alternation_sum.push((v.0, v.2));
                        } else {
                            continue 'pair_loop;
                        }
                    }

                    // We have a valid alternation sum and an affix, we will attempt to combine them.
                    // First we need to add all of the alternations together and see if the total is less
                    // than the max alternations.

                    // We use a HashMap to store the intermediate alternation blob because we need to keep
                    // track of the alternation ids.
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

                    // If the combined map has more than max_alternations, we can't use alternations on these
                    // two conjugations.
                    if combined_alternations.len() > input.max_alternations {
                        continue;
                    }

                    println!("{}:{} => {:?}", index, index2, combined_alternations);

                    // We need to make a new list of affices
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
                            .chars()
                            .map(|c| {
                                if c.is_ascii_digit() {
                                    // Get ID for *this* alternation
                                    let alternation_id = c.to_digit(10).unwrap() as usize;
                                    // Search through the combined alternations to find the matching one
                                    let new_alternation_id = combined_alternations
                                        .iter()
                                        .find(|(_, v)| v.0.contains(&(alternation_id, affix_id)))
                                        .unwrap()
                                        .1
                                         .1;
                                    // Convert the new alternation id to a char
                                    std::char::from_digit(new_alternation_id as u32, 10).unwrap()
                                } else {
                                    c
                                }
                            })
                            .collect();

                        new_affices.push(AffixSet {
                            root_id,
                            prefix: String::new(),
                            suffix: affix,
                        });
                    }

                    let mut alternations = Vec::new();
                    for (alternation, map) in combined_alternations {
                        alternations.push(vec![Alternation {
                            target_pattern: format!("\\{}", map.1),
                            replacement_pattern: alternation.0,
                        }]);
                        alternations.push(vec![Alternation {
                            target_pattern: format!("\\{}", map.1),
                            replacement_pattern: alternation.1,
                        }]);
                    }

                    for affix in &new_affices {
                        println!("{:?}", affix.suffix);
                    }

                    // We need to kick modifying the conjugations to outside the loop
                    // because of the iterators
                    action = Some((
                        index,
                        index2,
                        AffixConjugation {
                            affices: new_affices,
                            alternations,
                        },
                    ));

                    break 'outer;
                }
            }

            if let Some(v) = action {
                flag = true;
                conjugations.remove(v.0);
                let r = if v.0 < v.1 { v.1 - 1 } else { v.1 };
                conjugations[r] = v.2;
            }
        }
    }

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

    println!("{:?}", conjugations);
    println!("{}", table_string);

    todo!()
}

fn remove_duplicate_conjugations(conjugations: &mut Vec<AffixConjugation>) {
    // First we check for any duplicate conjugations
    // If we find any, we merge them together
    // We do this by checking if the affices are the same
    let mut flag = true;
    while flag {
        let mut index_to_delete = None;
        'outer: for (index, conjugation) in conjugations.iter().enumerate() {
            for (index2, conjugation2) in conjugations.iter().enumerate() {
                if index == index2 {
                    continue;
                }
                if conjugation.affices == conjugation2.affices {
                    // We have a duplicate conjugation
                    // We need to merge them together
                    index_to_delete = Some(index2);
                    break 'outer;
                }
            }
        }
        if let Some(index) = index_to_delete {
            conjugations.remove(index);
            flag = true;
        } else {
            flag = false;
        }
    }
}

/// Creates base conjugations; each word will have it's own conjugation.
/// Automatically finds the roots and does all of that.
fn create_base_conjugations(input: &ConjugatorInput) -> Vec<AffixConjugation> {
    let mut conjugations = Vec::new();

    // We start by finding the longest common root
    for word in &input.words {
        let roots = match root_searcher_helper(
            Vec::new(),
            input.max_interconjugation_roots - 1,
            word.to_vec(),
        ) {
            Some(roots) => roots,
            None => {
                // We have an irregular word
                // TODO: Handle irregular words
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
        let conjugation = AffixConjugation {
            affices,
            alternations: Vec::new(),
        };
        conjugations.push(conjugation);
    }
    conjugations
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
    affix1: &str,
    affix2: &str,
    remaining_alternations: usize,
    previous_alternations: Vec<(Option<char>, Option<char>)>,
) -> AlternationIntermediateResult {
    let mut iter1 = affix1.chars();
    let mut iter2 = affix2.chars();

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
            return Some((
                affix1.to_string(),
                affix2.to_string(),
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

        let alternation_result = {
            let mut alternations = previous_alternations.clone();
            let find_or_insert_result =
                find_or_insert(&mut alternations, (Some(char1), Some(char2)));
            let newstr = &format!("\\{}", find_or_insert_result.0);
            let new_remaining_alternations = if find_or_insert_result.1 {
                remaining_alternations
            } else if remaining_alternations > 0 {
                remaining_alternations - 1
            } else {
                // We can't make any more alternations
                return None;
            };

            // We need to add the escape sequences to the affices
            let new_affix1 = replace_nth_char(affix1, char_index1 - 1, newstr);
            let new_affix2 = replace_nth_char(affix2, char_index2 - 1, newstr);

            alternation_helper(
                &new_affix1,
                &new_affix2,
                new_remaining_alternations,
                alternations,
            )
        };

        let insertion_result = {
            let mut alternations = previous_alternations.clone();
            let find_or_insert_result = find_or_insert(&mut alternations, (Some(char1), None));
            let newstr = &format!("\\{}", find_or_insert_result.0);
            let new_remaining_alternations = if find_or_insert_result.1 {
                remaining_alternations
            } else if remaining_alternations > 0 {
                remaining_alternations - 1
            } else {
                // We can't make any more alternations
                return None;
            };

            // We need to add the escape sequences to the affices
            let new_affix1 = replace_nth_char(affix1, char_index1 - 1, newstr);
            let new_affix2 = insert_nth_char(affix2, char_index2 - 1, newstr);

            alternation_helper(
                &new_affix1,
                &new_affix2,
                new_remaining_alternations,
                alternations,
            )
        };

        let deletion_result = {
            let mut alternations = previous_alternations;
            let find_or_insert_result = find_or_insert(&mut alternations, (None, Some(char2)));
            let newstr = &format!("\\{}", find_or_insert_result.0);
            let new_remaining_alternations = if find_or_insert_result.1 {
                remaining_alternations
            } else if remaining_alternations > 0 {
                remaining_alternations - 1
            } else {
                // We can't make any more alternations
                return None;
            };

            // We need to add the escape sequences to the affices
            let new_affix1 = insert_nth_char(affix1, char_index1 - 1, newstr);
            let new_affix2 = replace_nth_char(affix2, char_index2 - 1, newstr);

            alternation_helper(
                &new_affix1,
                &new_affix2,
                new_remaining_alternations,
                alternations,
            )
        };

        let mut best_result = None;
        let mut best_alternations = usize::MAX;

        if let Some(result) = alternation_result {
            if result.2.len() < best_alternations {
                best_alternations = result.2.len();
                best_result = Some(result);
            }
        }

        if let Some(result) = insertion_result {
            if result.2.len() < best_alternations {
                best_alternations = result.2.len();
                best_result = Some(result);
            }
        }

        if let Some(result) = deletion_result {
            if result.2.len() < best_alternations {
                // No need to update best_alternations because we're done
                best_result = Some(result);
            }
        }

        return best_result;
    }
}

/// Replaces the nth char with newstr
fn replace_nth_char(s: &str, idx: usize, newstr: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == idx {
                newstr.to_owned()
            } else {
                String::from(c)
            }
        })
        .collect()
}

/// Inserts newstr *before* the nth char
fn insert_nth_char(s: &str, idx: usize, newstr: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == idx {
                format!("{}{}", newstr, c)
            } else {
                String::from(c)
            }
        })
        .collect()
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
