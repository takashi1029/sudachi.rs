use nom::{le_i32, le_u16, le_u32, le_u8};

pub struct WordInfos<'a> {
    bytes: &'a [u8],
    offset: usize,
    word_size: u32,
}

impl<'a> WordInfos<'a> {
    pub fn new(bytes: &'a [u8], offset: usize, word_size: u32) -> WordInfos {
        WordInfos {
            bytes,
            offset,
            word_size,
        }
    }

    pub fn get_word_info(&self, word_id: usize) -> Option<WordInfo>  {
        if self.offset + (4 * word_id) > self.bytes.len() {
            return None;
        }
        let index = match le_u32(&self.bytes[self.offset + (4 * word_id)..]) {
            Ok(n) => n.1 as usize, // wordIdToOffset()
            Err(err) => return None
        };
        let mut word_info = word_info_parser(self.bytes, index).unwrap().1;

        // TODO: can we set dictionary_form within the word_info_parser?
        let dfwi = word_info.dictionary_form_word_id;
        if (dfwi >= 0) & (dfwi != word_id as i32) {
            word_info.dictionary_form = match self.get_word_info(dfwi as usize) {
                Some(result) => result.surface,
                None => return None
            }
        };

        Some(word_info)
    }

    // TODO: is_valid_split()
}

named!(
    utf16_string<&[u8], String>,
    do_parse!(
        length: le_u8 >>
        v: count!(le_u16, length as usize) >>

        (match String::from_utf16(&v) {
            Ok(n) => n,
            Err(err) => "".to_string()
        })
    )
);

named_args!(
    word_info_parser(index: usize)<&[u8], WordInfo>,
    do_parse!(
        _seek: take!(index) >>
        surface: utf16_string >>
        head_word_length: le_u8 >>
        pos_id: le_u16 >>
        normalized_form: utf16_string >>
        dictionary_form_word_id: le_i32 >>
        reading_form: utf16_string >>

        a_unit_split_count: le_u8 >>
        a_unit_split: count!(le_u32, a_unit_split_count as usize) >>

        b_unit_split_count: le_u8 >>
        b_unit_split: count!(le_u32, b_unit_split_count as usize) >>

        word_structure_count: le_u8 >>
        word_structure: count!(le_u32, word_structure_count as usize) >>

        (WordInfo{
            normalized_form: match normalized_form.as_str() {
                "" => surface.clone(),
                _ => normalized_form
            },
            dictionary_form: surface.clone(), // TODO: can we set this within the parser?
            surface, // after normalized_form and dictionary_form, as it may be cloned there
            head_word_length,
            pos_id,
            reading_form,
            a_unit_split,
            b_unit_split,
            word_structure,
            dictionary_form_word_id,
        })
    )
);

#[derive(Debug)]
pub struct WordInfo {
    pub surface: String,
    head_word_length: u8,
    pub pos_id: u16,
    pub normalized_form: String,
    pub reading_form: String,
    pub a_unit_split: Vec<u32>,
    pub b_unit_split: Vec<u32>,
    pub word_structure: Vec<u32>,
    pub dictionary_form_word_id: i32,
    pub dictionary_form: String,
}
