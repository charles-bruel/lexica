use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fmt;

use crate::manual_ux::project::Project;
use crate::{priority_queue::PriorityQueue, websocket_handler::WebSocketResponse};

pub type PredicateDef = (Vec<Box<dyn Predicate>>, Vec<(usize, u64)>);
pub type ResultDef = (Vec<Box<dyn Result>>, Vec<usize>);

pub struct Program {
    pub features: Vec<Feature>,
    pub diacritics: Vec<Diacritic>,
    pub rules: Vec<Rule>,
    pub subroutines: HashMap<String, Vec<Rule>>,
    pub labels: HashMap<String, usize>,
    pub names_to_idx: HashMap<String, u32>,
    pub idx_to_features: HashMap<u32, Feature>,
    pub features_to_idx: HashMap<String, (u32, usize)>,
    pub symbol_to_letter: HashMap<String, (Letter, u64)>,
    pub letter_to_symbol: HashMap<Letter, String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Word {
    pub letters: Vec<Letter>,
    pub syllables: Vec<SyllableDefinition>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SyllableDefinition {
    pub start: usize,
    pub end: usize,
}

pub struct ProgramCreationContext {
    pub rule_line_defs: HashMap<usize, u32>,
}

#[derive(Clone)]
pub enum Feature {
    SwitchType(SwitchType),
    FeatureDef(FeatureDef),
}

#[derive(Clone)]
pub struct SwitchType {
    pub start_byte: u8,
    pub tot_length: u8,
    pub self_length: u8,
    pub features: Vec<Vec<Feature>>,
    pub name: String,
    pub option_names: Vec<String>,
    pub id: u32,
    pub validation_mask: u64,
    pub validation_key: u64,
}

#[derive(Clone)]
pub struct FeatureDef {
    pub start_byte: u8,
    pub length: u8,
    pub name: String,
    pub option_names: Vec<String>,
    pub is_bool: bool,
    pub id: u32,
    pub validation_mask: u64,
    pub validation_key: u64,
}

impl Word {
    pub fn get_syllable<'a>(&'a mut self, syllable: SyllableDefinition) -> Vec<&'a mut Letter> {
        let mut result: Vec<&'a mut Letter> = Vec::with_capacity(syllable.len());
        for x in self.letters[syllable.start..syllable.end + 1].iter_mut() {
            result.push(x);
        }
        result
    }

    pub fn len(&self) -> usize {
        self.letters.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, index: usize, element: Letter) {
        self.letters.insert(index, element);
        for x in &mut self.syllables {
            if x.start > index {
                x.start += 1;
            }
            if x.end > index || x.end == self.letters.len() - 1 {
                x.end += 1;
            }
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.letters.remove(index);
        for x in &mut self.syllables {
            if x.start > index {
                x.start -= 1;
            }
            if x.end > index {
                x.end -= 1;
            }
        }
        //Check for syllables made 0 length
        let mut i = 0;
        while i < self.syllables.len() {
            if self.syllables[i].start == self.syllables[i].end {
                self.syllables.remove(i);
            } else {
                //If we removed a syllable, we need to subtract one from i
                //Doing it here removes chance of underflow
                i += 1;
            }
        }
    }
}

impl SyllableDefinition {
    pub fn len(&self) -> usize {
        1 + self.end - self.start
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl core::ops::Index<usize> for Word {
    type Output = Letter;
    fn index(&self, index: usize) -> &Self::Output {
        &self.letters[index]
    }
}

impl core::ops::IndexMut<usize> for Word {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.letters[index]
    }
}

impl Feature {
    pub fn start_byte(&self) -> u8 {
        match self {
            Feature::SwitchType(data) => data.start_byte,
            Feature::FeatureDef(data) => data.start_byte,
        }
    }
    pub fn length(&self) -> u8 {
        match self {
            Feature::SwitchType(data) => data.self_length,
            Feature::FeatureDef(data) => data.length,
        }
    }
    pub fn tot_length(&self) -> u8 {
        match self {
            Feature::SwitchType(data) => data.tot_length,
            Feature::FeatureDef(data) => data.length,
        }
    }
    pub fn name(&self) -> String {
        match self {
            Feature::SwitchType(data) => data.name.clone(),
            Feature::FeatureDef(data) => data.name.clone(),
        }
    }
    pub fn id(&self) -> u32 {
        match self {
            Feature::SwitchType(data) => data.id,
            Feature::FeatureDef(data) => data.id,
        }
    }
    pub fn validation_key(&self) -> u64 {
        match self {
            Feature::SwitchType(data) => data.validation_key,
            Feature::FeatureDef(data) => data.validation_key,
        }
    }
    pub fn validation_mask(&self) -> u64 {
        match self {
            Feature::SwitchType(data) => data.validation_mask,
            Feature::FeatureDef(data) => data.validation_mask,
        }
    }
    pub fn clone_light(&self) -> Feature {
        match self {
            Feature::SwitchType(data) => {
                let mut temp = data.clone();
                temp.features = Vec::new();
                Feature::SwitchType(temp)
            }
            Feature::FeatureDef(_) => self.clone(),
        }
    }
    pub fn validate(&self, letter: &Letter) -> bool {
        let validation = match self {
            Feature::SwitchType(data) => (data.validation_key, data.validation_mask),
            Feature::FeatureDef(data) => (data.validation_key, data.validation_mask),
        };
        let value = letter.value;
        value & validation.1 == validation.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Letter {
    pub value: u64,
}

impl Letter {
    pub fn get_symbol(&self, program: &Program) -> std::result::Result<String, ApplicationError> {
        let temp = program.letter_to_symbol.get(self);
        match temp {
            Some(result) => Ok(result.to_string()),
            None => {
                let mut queue: PriorityQueue<(u64, u64, &str), i8> = PriorityQueue::new();
                queue.push((self.value, self.value, ""), 0);
                let mut completed_nodes: HashMap<u64, (u64, &str)> = HashMap::new();
                let mut depth: u16 = 0;
                while depth < 1024 {
                    if queue.is_empty() {
                        return Err(ApplicationError::IntoConversionError(format!(
                            "Could not find matching symbol for {:#066b}",
                            self.value
                        )));
                    }
                    let ((value, prev_node, current_symbol), priority) = queue.pop().unwrap();

                    for diacritic in &program.diacritics {
                        if diacritic.mask & value == diacritic.mod_key {
                            let new_value = (value & !diacritic.mask) | diacritic.key;
                            queue.push((new_value, value, &diacritic.diacritic), priority - 1);
                        }
                    }

                    if prev_node != value {
                        completed_nodes.insert(value, (prev_node, current_symbol));
                    }

                    let letter = Letter { value };
                    if program.letter_to_symbol.contains_key(&letter) {
                        let mut result = letter.get_symbol(program)?;
                        let mut current_node = value;
                        let mut depth2: u16 = 0;
                        while depth2 < 128 {
                            let (new_node, symbol) = match completed_nodes.get(&current_node) {
                                Some(val) => val,
                                None => return Ok(result),
                            };
                            current_node = *new_node;
                            result.push_str(symbol);
                            depth2 += 1;
                        }
                        return Err(ApplicationError::IntoConversionError(format!(
                            "Could not find matching symbol for {:#066b}",
                            self.value
                        )));
                    }

                    depth += 1
                }

                Err(ApplicationError::IntoConversionError(format!(
                    "Could not find matching symbol for {:#066b}",
                    self.value
                )))
            }
        }
    }
}

pub trait Predicate {
    fn validate(&self, word: &Word, position: usize) -> bool;
}

pub trait Result {
    fn transform(&self, input: &Letter) -> Option<Letter>;
}

pub struct Transformation {
    pub predicate: Vec<Box<dyn Predicate>>,
    pub result: Vec<Box<dyn Result>>,
    pub predicate_captures: Vec<(usize, u64)>,
    pub result_captures: Vec<usize>,
}

pub struct RuleByte {
    pub transformations: Vec<Transformation>,
    pub enviorment: Enviorment,
    pub num_captures: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JumpCondition {
    PrevMod,
    Flag,
    Unconditional,
}

pub enum Rule {
    TransformationRule {
        bytes: Vec<RuleByte>,
        flags: u16,
        name: String,
    },
    CallSubroutine {
        name: String,
    },
    JumpSubRoutine {
        name: String,
        condition: JumpCondition,
        inverted: bool,
    },
    Detect {
        predicate: Vec<Box<dyn Predicate>>,
        enviorment: Enviorment,
    },
}

pub struct EnviormentPredicate {
    pub predicate: Box<dyn Predicate>,
    pub min_quant: u8,
    pub max_quant: u8,
}

pub struct Enviorment {
    pub ante: Vec<EnviormentPredicate>,
    pub post: Vec<EnviormentPredicate>,
    pub ante_word_boundary: bool,
    pub post_word_boundary: bool,
    pub inverted: bool,
}

pub struct Diacritic {
    pub diacritic: String,
    pub mask: u64,
    pub key: u64,
    pub mod_key: u64,
}

#[derive(PartialEq)]
pub enum Ordering {
    Forward,
    Reverse,
}

static mut FEATURE_ID_TRACKER: u32 = 0;
//Technically unsafe but the program generation will not occur in threads
fn get_id() -> u32 {
    unsafe {
        FEATURE_ID_TRACKER += 1;
        FEATURE_ID_TRACKER
    }
}

pub fn create_empty_program() -> Program {
    Program {
        features: Vec::new(),
        diacritics: Vec::new(),
        rules: Vec::new(),
        subroutines: HashMap::new(),
        labels: HashMap::new(),
        names_to_idx: HashMap::new(),
        idx_to_features: HashMap::new(),
        features_to_idx: HashMap::new(),
        letter_to_symbol: HashMap::new(),
        symbol_to_letter: HashMap::new(),
    }
}

pub fn create_program_creation_context() -> ProgramCreationContext {
    ProgramCreationContext {
        rule_line_defs: HashMap::new(),
    }
}

pub fn create_feature_def_bool(
    name: String,
    negative_option: String,
    positive_option: String,
) -> FeatureDef {
    let id = get_id();
    FeatureDef {
        start_byte: 0,
        length: 1,
        name,
        option_names: vec![negative_option, positive_option],
        is_bool: true,
        id,
        validation_key: 0,
        validation_mask: 0,
    }
}

pub fn create_feature_def(name: String, option_names: Vec<String>) -> FeatureDef {
    let len = f64::from((option_names.len() + 1) as u32).log2().ceil() as u8;
    let id = get_id();
    FeatureDef {
        start_byte: 0,
        length: len,
        name,
        option_names,
        is_bool: false,
        id,
        validation_key: 0,
        validation_mask: 0,
    }
}

pub fn create_switch_type(
    name: String,
    option_names: Vec<String>,
    features: Vec<Vec<Feature>>,
) -> SwitchType {
    let len = f64::from((option_names.len() + 1) as u32).log2().ceil() as u8;
    let id = get_id();
    SwitchType {
        start_byte: 0,
        self_length: len,
        tot_length: 0,
        features,
        name,
        option_names,
        id,
        validation_key: 0,
        validation_mask: 0,
    }
}

pub fn create_diacritic(diacritic: String, mask: u64, key: u64, mod_key: u64) -> Diacritic {
    Diacritic {
        diacritic,
        mask,
        key,
        mod_key,
    }
}

impl Program {
    pub fn print_structure(&self) {
        print_structure_recurse(&self.features, 0);
    }
}

fn print_structure_recurse(features: &Vec<Feature>, level: u8) {
    for f in features {
        let whitespace: String = String::from_utf8(vec![b'\t'; usize::from(level)])
            .ok()
            .unwrap();
        match f {
            Feature::SwitchType(data) => {
                print!("{}", whitespace);
                println!(
                    "Name: {0}, Start byte: {1}, Self Length: {2}, Total Length {3}",
                    data.name, data.start_byte, data.self_length, data.tot_length
                );
                let mut i: usize = 0;
                while i < data.option_names.len() {
                    if !data.features[i].is_empty() {
                        print!("{}", whitespace);
                        println!("Option: {}:", data.option_names[i]);
                        print_structure_recurse(&data.features[i], level + 1);
                    }
                    i += 1;
                }
            }
            Feature::FeatureDef(data) => {
                print!("{}", whitespace);
                println!(
                    "Name: {0}, Start byte: {1}, Length: {2}",
                    data.name, data.start_byte, data.length
                );
            }
        }
    }
}

pub fn create_rule_byte(
    predicate: PredicateDef,
    result: ResultDef,
    enviorment: Enviorment,
) -> std::result::Result<RuleByte, ConstructorError> {
    let mut num_captures: usize = 0;

    for x in &predicate.1 {
        if num_captures < x.0 {
            num_captures = x.0;
        }
    }

    for x in &result.1 {
        if *x > num_captures {
            return Err(create_constructor_error_empty(
                "More output captures than input captures",
                line!(),
                ConstructorErrorType::MalformedDefinition,
            ));
        }
    }

    Ok(RuleByte {
        transformations: vec![Transformation {
            predicate: predicate.0,
            result: result.0,
            predicate_captures: predicate.1,
            result_captures: result.1,
        }],
        enviorment,
        num_captures: num_captures + 1,
    })
}

pub fn create_multi_rule_byte(
    predicate: Vec<PredicateDef>,
    result: Vec<ResultDef>,
    enviorment: Enviorment,
) -> std::result::Result<RuleByte, ConstructorError> {
    if predicate.len() != result.len() {
        return Err(create_constructor_error_empty(
            "Predicate and result have different lengths",
            line!(),
            ConstructorErrorType::MalformedDefinition,
        ));
    }
    let mut transformations: Vec<Transformation> = Vec::new();

    let mut i: usize = 0;
    while i < predicate.len() {
        transformations.push(Transformation {
            predicate: Vec::new(),
            result: Vec::new(),
            predicate_captures: Vec::new(),
            result_captures: Vec::new(),
        });
        i += 1;
    }

    let mut num_captures: usize = 0;

    i = 0;
    for p in predicate {
        transformations[i].predicate = p.0;

        for x in &p.1 {
            if num_captures < x.0 {
                num_captures = x.0;
            }
        }
        transformations[i].predicate_captures = p.1;

        i += 1;
    }

    i = 0;
    for r in result {
        transformations[i].result = r.0;

        for x in &r.1 {
            if *x > num_captures {
                return Err(create_constructor_error_empty(
                    "More output captures than input captures",
                    line!(),
                    ConstructorErrorType::MalformedDefinition,
                ));
            }
        }
        transformations[i].result_captures = r.1;

        i += 1;
    }

    Ok(RuleByte {
        transformations,
        enviorment,
        num_captures: num_captures + 1,
    })
}

pub fn create_transformation_rule(name: String, bytes: Vec<RuleByte>, flags: u16) -> Rule {
    Rule::TransformationRule { bytes, flags, name }
}

pub fn create_subroutine_call_rule(name: String) -> Rule {
    Rule::CallSubroutine { name }
}

pub fn create_jump_rule(name: String, condition: JumpCondition, inverted: bool) -> Rule {
    Rule::JumpSubRoutine {
        name,
        condition,
        inverted,
    }
}

pub fn create_detect_rule(predicate: Vec<Box<dyn Predicate>>, enviorment: Enviorment) -> Rule {
    Rule::Detect {
        predicate,
        enviorment,
    }
}

pub fn create_empty_enviorment() -> Enviorment {
    Enviorment {
        ante: Vec::new(),
        post: Vec::new(),
        ante_word_boundary: false,
        post_word_boundary: false,
        inverted: false,
    }
}

pub fn create_enviorment(
    ante: Vec<EnviormentPredicate>,
    post: Vec<EnviormentPredicate>,
    ante_word_boundary: bool,
    post_word_boundary: bool,
    inverted: bool,
) -> Enviorment {
    Enviorment {
        ante,
        post,
        ante_word_boundary,
        post_word_boundary,
        inverted,
    }
}

pub fn create_enviorment_predicate_single(predicate: Box<dyn Predicate>) -> EnviormentPredicate {
    EnviormentPredicate {
        predicate,
        min_quant: 1,
        max_quant: 1,
    }
}

pub fn create_enviorment_predicate(
    predicate: Box<dyn Predicate>,
    min: u8,
    max: u8,
) -> EnviormentPredicate {
    EnviormentPredicate {
        predicate,
        min_quant: min,
        max_quant: max,
    }
}

pub fn to_string(program: &Program, word: Word) -> std::result::Result<String, ApplicationError> {
    let mut result = String::from("");
    for (index, l) in word.letters.iter().enumerate() {
        for x in &word.syllables {
            if (x.start == index || x.end == index) && index != 0 && index != word.len() {
                result += ".";
                //We only place on dot per syllable boundary, but two will match;
                //the end of the prev and the start of the next
                break;
            }
        }
        result += &l.get_symbol(program)?;
    }
    Ok(result)
}

pub struct ExecutionContext {
    pub instruction_ptr: usize,
    pub result: Word,
    pub mod_flag: bool,
    pub flag_flag: bool,
    pub jump_flag: bool,
}

pub fn create_execution_context(result: &Word) -> ExecutionContext {
    ExecutionContext {
        instruction_ptr: 0,
        result: result.clone(),
        mod_flag: false,
        flag_flag: false,
        jump_flag: false,
    }
}

// TODO: Move out of sc folder
pub struct ThreadContext {
    pub project: Project,
    pub queued_extra_messages: VecDeque<WebSocketResponse>, // TODO: Unify extra and regular response messages
}

pub fn create_thread_context() -> ThreadContext {
    ThreadContext {
        project: Project {
            tables: Vec::new(),
            programs: HashMap::new(),
        },
        queued_extra_messages: VecDeque::new(),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ApplicationError {
    IntoConversionError(String),
    OutofConversionError(String),
    InternalError(String),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApplicationError::IntoConversionError(v) => write!(f, "IntoConversionError({})", v),
            ApplicationError::OutofConversionError(v) => write!(f, "OutofConversionError({})", v),
            ApplicationError::InternalError(v) => write!(f, "InternalError({})", v),
        }
    }
}

pub fn create_constructor_error<S>(
    error_message: S,
    line_contents: String,
    line_number_user_program: u32,
    line_number_code: u32,
    error_type: ConstructorErrorType,
) -> ConstructorError
where
    S: Into<String>,
{
    ConstructorError {
        error_message: error_message.into(),
        line_contents,
        line_number_user_program: LineNumberInformation::Raw(line_number_user_program),
        line_number_code,
        error_type,
    }
}

pub fn create_constructor_error_offset(
    error_message: String,
    line_contents: String,
    line_number_user_program: i8,
    line_number_code: u32,
    error_type: ConstructorErrorType,
) -> ConstructorError {
    ConstructorError {
        error_message,
        line_contents,
        line_number_user_program: LineNumberInformation::Offset(line_number_user_program),
        line_number_code,
        error_type,
    }
}

pub fn create_constructor_error_empty<S>(
    error_message: S,
    line_number_code: u32,
    error_type: ConstructorErrorType,
) -> ConstructorError
where
    S: Into<String>,
{
    ConstructorError {
        error_message: error_message.into(),
        line_contents: String::from(""),
        line_number_user_program: LineNumberInformation::Undetermined,
        line_number_code,
        error_type,
    }
}

pub fn create_word_syllables(letters: Vec<Letter>, syllables: Vec<SyllableDefinition>) -> Word {
    Word { letters, syllables }
}

pub fn create_empty_word() -> Word {
    Word {
        letters: Vec::new(),
        syllables: Vec::new(),
    }
}

pub fn create_word(letters: Vec<Letter>) -> Word {
    Word {
        letters,
        syllables: Vec::new(),
    }
}

pub fn create_syllable_definition(
    start: usize,
    end: usize,
) -> std::result::Result<SyllableDefinition, ApplicationError> {
    if start > end {
        return Err(ApplicationError::InternalError(String::from(
            "Syllable start is after end",
        )));
    }

    Ok(SyllableDefinition { start, end })
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConstructorError {
    pub error_message: String,
    pub line_contents: String,
    pub line_number_user_program: LineNumberInformation,
    pub line_number_code: u32,
    pub error_type: ConstructorErrorType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ConstructorErrorType {
    UnknownCommandError,
    HangingSection,
    MalformedDefinition,
    MissingNode,
    FeatureOverflow,
    MissingSymbol,
    InvalidFeature,
    MissingFeature,
    ParseError,
    MissingSubroutine,
    MissingLabel,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum LineNumberInformation {
    Undetermined,
    Offset(i8),
    Raw(u32),
}

impl fmt::Display for ConstructorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error_type_name = match self.error_type {
            ConstructorErrorType::UnknownCommandError => "UnknownCommandError",
            ConstructorErrorType::HangingSection => "HangingSection",
            ConstructorErrorType::MalformedDefinition => "MalformedDefinition",
            ConstructorErrorType::MissingNode => "MissingNode",
            ConstructorErrorType::FeatureOverflow => "FeatureOverflow",
            ConstructorErrorType::MissingSymbol => "MissingSymbol",
            ConstructorErrorType::InvalidFeature => "InvalidFeature",
            ConstructorErrorType::MissingFeature => "MissingFeature",
            ConstructorErrorType::ParseError => "ParseError",
            ConstructorErrorType::MissingSubroutine => "MissingSubroutine",
            ConstructorErrorType::MissingLabel => "MissingLabel",
        };
        write_constructor_error(
            f,
            error_type_name,
            &self.error_message,
            &self.line_contents,
            self.line_number_user_program,
            self.line_number_code,
        )
        .expect("Error formatting error message");
        Ok(())
    }
}

fn write_constructor_error(
    formatter: &mut fmt::Formatter,
    type_message: &'static str,
    error_message: &String,
    line_contents: &String,
    line_number_user_program: LineNumberInformation,
    line_number_code: u32,
) -> std::result::Result<(), std::fmt::Error> {
    let line_number: u32 = match line_number_user_program {
        LineNumberInformation::Undetermined => {
            print!("Line number for error has not been determined, giving back 0.");
            0
        }
        LineNumberInformation::Offset(_) => {
            print!("Line number for error has not been fully determined, giving back 0.");
            0
        }
        LineNumberInformation::Raw(v) => v,
    };

    write!(
        formatter,
        "{}({}: Line {}) on line {}; {}",
        type_message, error_message, line_number_code, line_number, line_contents
    )
}
