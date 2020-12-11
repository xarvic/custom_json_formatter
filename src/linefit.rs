use serde_json::ser::{Formatter, CharEscape};
use std::io;
use std::ops::RangeInclusive;

#[derive(Copy, Clone, Eq, PartialEq)]
enum ElementType {
    OpenBracket,
    Element,
    ObjectKey,
    CloseBracket,
}

impl ElementType {
    pub fn level(&self) -> isize {
        match self {
            ElementType::OpenBracket => 1,
            ElementType::Element => 0,
            ElementType::ObjectKey => 0,
            ElementType::CloseBracket => -1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Element {
    end: usize,
    complete_line_length: usize,
    element_type: ElementType,
}



pub struct CompactPrettyFormatter<'a> {
    //the longest allowed String
    line_break_with: usize,
    //
    indent: &'a str,
    //the impact of the indent to the line_length
    indent_impact: usize,

    //---------------Changing---------------------

    // write type
    started_string: bool,
    started_key: bool,

    // count of unmatched opened brackets ('{' or '[') which doesnt appear in elements
    written_unmatched_brackets: usize,

    // count of unmatched opened brackets ('{' or '[') in elements which doesnt have a counterpart
    // inside of elements
    // this value is negative if elements contains more closing than opening brackets!
    unwritten_unmatched_brackets: isize,
    // Unwritten elements
    elements: Vec<Element>,
    // Unwritten data
    cached_data: Vec<u8>,

    current_element_end: usize,
    current_length: usize,

    // notes if the current line contains already data => dont write indents again!
    line_started: bool,

    written_buffer_length: usize,
    written_line_length: usize,
}

impl<'a> CompactPrettyFormatter<'a> {
    pub fn new(line_break_with: usize, indent: &'a str, include_indent: bool) -> Self {
        let mut this = Self{
            line_break_with,
            indent,
            indent_impact: 0,
            started_string: false,
            started_key: false,
            written_unmatched_brackets: 0,
            unwritten_unmatched_brackets: 0,
            elements: vec![],
            cached_data: vec![],
            current_element_end: 0,
            current_length: 0,
            line_started: false,
            written_buffer_length: 0,
            written_line_length: 0
        };
        let impact = if include_indent {
            this.display_length(indent)
        } else {
            0
        };
        this.indent_impact = impact;
        this
    }
    pub fn start_string(&mut self, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        if !self.started_string {
            self.started_string = true;
            self.write_part("\"", writer)?;
            Ok(())
        } else {
            panic!("started string before ending the last!")
        }
    }
    pub fn end_string(&mut self, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        if self.started_string {
            self.started_string = false;
            self.write("\"", ElementType::Element, writer)?;
            Ok(())
        } else {
            panic!("String to close was´nt opened!")
        }
    }
    pub fn start_key(&mut self, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        if !self.started_key {
            self.started_key = true;
            Ok(())
        } else {
            panic!("started object key before ending the last!")
        }
    }
    pub fn end_key(&mut self, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        if self.started_key {
            self.started_string = false;
            self.write(":", ElementType::ObjectKey, writer)?;
            Ok(())
        } else {
            panic!("object key to close was´nt opened!")
        }
    }

    fn end_of_structure(&self, index: usize) -> Option<usize> {
        let mut elements = self.elements.iter().zip(index..);
        if elements.next().unwrap().0.element_type == ElementType::Element {
            Some(index)
        } else {
            let mut open_brackets = 1;

            for (element, index) in elements {
                open_brackets += element.element_type.level();
                if open_brackets == 0 {
                    return Some(index)
                }
            }
            None
        }
    }
    fn line_start_of(&self, index: usize) -> usize {
        if index == 0 {
            self.written_line_length
        } else {
            self.elements[index - 1].complete_line_length
        }
    }
    fn buffer_start_of(&self, index: usize) -> usize {
        if index == 0 {
            self.written_buffer_length
        } else {
            self.elements[index - 1].end
        }
    }

    // returns the indices the the elements which describe the first full structure
    // ('{...}' or '[...]') that is part of a structure which is known to not fit in display width
    // returns None if we dont know if a unclosed structure will maybe fit, Some otherwise
    //
    fn first_full_structure(&self) -> Option<RangeInclusive<usize>> {
        let elements = self.elements.iter().enumerate();

        let mut opened_brackets = self.written_unmatched_brackets;

        for (index, element) in elements {
            if let Some(end) = self.end_of_structure(index) {
                //Structure fits!
                if self.line_break_with > self.indent_impact * opened_brackets + self.elements[end].complete_line_length - self.line_start_of(index) || index == end {
                    return Some(index..=end)
                }
            }
            opened_brackets = (opened_brackets as isize + element.element_type.level()) as usize;
        }
        None
    }


    pub fn display_length(&self, string: &str) -> usize {
        string.len()
    }
    pub fn write_indents(&self, count: usize, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        for i in 0..count {
            writer.write(self.indent.as_bytes())?;
        }
        Ok(())
    }

    pub fn write_in_line(&mut self, elements: usize, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        self.elements.iter().take(elements).for_each(|element|{

        });
        Ok(())
    }

    pub fn write_back_overflowing_elements(&mut self, writer: &mut (impl ?Sized + io::Write)) -> io::Result<()> {
        Ok(())
    }

    pub fn write_part<W: ?Sized + io::Write>(&mut self, data: &str, writer: &mut W) -> io::Result<()> {
        self.current_length += self.display_length(data);

        self.cached_data.extend_from_slice(data.as_bytes());

        //write if the current length exceeds the maximal width
        if self.current_length as isize + (self.unwritten_unmatched_brackets) * self.indent_impact as isize > self.line_break_with as isize {
            self.write_back_overflowing_elements(writer)?;
        }

        Ok(())
    }

    fn write<W: ?Sized + io::Write>(&mut self, data: &str, element_type: ElementType, writer: &mut W) -> io::Result<()> {
        let result = self.write_part(data, writer);

        if !self.started_key && !self.started_string {
            let open_impact = match element_type {
                ElementType::OpenBracket => 1,
                ElementType::Element => 0,
                ElementType::ObjectKey => 0,
                ElementType::CloseBracket => -1,
            };

            self.elements.push(Element {
                end: self.current_element_end,
                complete_line_length: self.current_length,
                element_type
            });

            //The next Element will start at the current end of the buffer
            self.current_element_end = self.cached_data.len();

            if self.written_unmatched_brackets as isize + self.unwritten_unmatched_brackets == 0 {
                // We are finished => write back all elements, they will fit in one line, if not
                // Self::write_part would have written the the part which does##nt fit!

            }
        }
        result
    }
}

impl<'a> Formatter for CompactPrettyFormatter<'a> {
    /// Writes a `null` value to the specified writer.
    #[inline]
    fn write_null<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write("null", ElementType::Element, writer)
    }

    /// Writes a `true` or `false` value to the specified writer.
    #[inline]
    fn write_bool<W>(&mut self, writer: &mut W, value: bool) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let s = if value {
            "true"
        } else {
            "false"
        };
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i8<W>(&mut self, writer: &mut W, value: i8) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i16<W>(&mut self, writer: &mut W, value: i16) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i32<W>(&mut self, writer: &mut W, value: i32) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `-123` to the specified writer.
    #[inline]
    fn write_i64<W>(&mut self, writer: &mut W, value: i64) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u8<W>(&mut self, writer: &mut W, value: u8) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u16<W>(&mut self, writer: &mut W, value: u16) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u32<W>(&mut self, writer: &mut W, value: u32) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes an integer value like `123` to the specified writer.
    #[inline]
    fn write_u64<W>(&mut self, writer: &mut W, value: u64) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = itoa::Buffer::new();
        let s = buffer.format(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes a floating point value like `-31.26e+12` to the specified writer.
    #[inline]
    fn write_f32<W>(&mut self, writer: &mut W, value: f32) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes a floating point value like `-31.26e+12` to the specified writer.
    #[inline]
    fn write_f64<W>(&mut self, writer: &mut W, value: f64) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        self.write(s, ElementType::Element, writer)
    }

    /// Writes a number that has already been rendered to a string.
    #[inline]
    fn write_number_str<W>(&mut self, writer: &mut W, value: &str) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write(value, ElementType::Element, writer)
    }

    /// Called before each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn begin_string<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.start_string(writer)
    }

    /// Called after each series of `write_string_fragment` and
    /// `write_char_escape`.  Writes a `"` to the specified writer.
    #[inline]
    fn end_string<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.end_string(writer)

    }

    /// Writes a string fragment that doesn't need any escaping to the
    /// specified writer.
    #[inline]
    fn write_string_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write_part(fragment, writer)
    }

    /// Writes a character escape code to the specified writer.
    #[inline]
    fn write_char_escape<W>(&mut self, writer: &mut W, char_escape: CharEscape) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        use self::CharEscape::*;

        let s = match char_escape {
            Quote => "\\\"",
            ReverseSolidus => "\\\\",
            Solidus => "\\/",
            Backspace => "\\b",
            FormFeed => "\\f",
            LineFeed => "\\n",
            CarriageReturn => "\\r",
            Tab => "\\t",
            AsciiControl(byte) => {
                todo!()
                /*static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                let bytes = &[
                    b'\\',
                    b'u',
                    b'0',
                    b'0',
                    HEX_DIGITS[(byte >> 4) as usize],
                    HEX_DIGITS[(byte & 0xF) as usize],
                ];
                return writer.write_all(bytes);*/
            }
        };

        self.write_part(s, writer)
    }

    /// Called before every array.  Writes a `[` to the specified
    /// writer.
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write("[", ElementType::OpenBracket, writer)
    }

    /// Called after every array.  Writes a `]` to the specified
    /// writer.
    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write("]", ElementType::CloseBracket, writer)
    }

    /// Called before every array value.  Writes a `,` if needed to
    /// the specified writer.
    #[inline]
    fn begin_array_value<W>(&mut self, _: &mut W, _: bool) -> io::Result<()>
        where W: ?Sized + io::Write, {Ok(())}

    /// Called after every array value.
    #[inline]
    fn end_array_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
        where W: ?Sized + io::Write, { Ok(()) }

    /// Called before every object.  Writes a `{` to the specified
    /// writer.
    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write("{", ElementType::OpenBracket, writer)
    }

    /// Called after every object.  Writes a `}` to the specified
    /// writer.
    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.write("[", ElementType::CloseBracket, writer)
    }

    /// Called before every object key.
    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.start_key(writer)
    }

    /// Called after every object key.  A `:` should be written to the
    /// specified writer by either this method or
    /// `begin_object_value`.
    #[inline]
    fn end_object_key<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.end_key(writer)
    }

    /// Called before every object value.  A `:` should be written to
    /// the specified writer by either this method or
    /// `end_object_key`.
    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        writer.write_all(b":")
    }

    /// Called after every object value.
    #[inline]
    fn end_object_value<W>(&mut self, _writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        Ok(())
    }

    /// Writes a raw JSON fragment that doesn't need any escaping to the
    /// specified writer.
    #[inline]
    fn write_raw_fragment<W>(&mut self, writer: &mut W, fragment: &str) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        writer.write_all(fragment.as_bytes())
    }
}