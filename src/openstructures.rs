use serde_json::ser::Formatter;
use std::io;
use std::io::Write;

pub struct OpenStructures<'a>{
    indent: &'a str,
    fold_after: u32,
    open: u32,
}

impl<'a> OpenStructures<'a> {
    pub fn new(indent: &'a str, fold_after: u32) -> Self {
        OpenStructures {
            indent,
            fold_after,
            open: 0
        }
    }
    /// provides the needed amount of indents basend on Self::open
    /// In opening structure methods this should get called after increasing open
    /// In closing structure methods this should get called before decreasing open
    fn print_indents(&self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        for _ in 0..self.open {
            writer.write(self.indent.as_bytes())?;
        }
        Ok(())
    }
    fn print_indents_below(&self, writer: &mut (impl Write + ?Sized)) -> io::Result<()> {
        for _ in 0..(self.open-1) {
            writer.write(self.indent.as_bytes())?;
        }
        Ok(())
    }
    fn is_open(&self) -> bool {
        self.open <= self.fold_after
    }
}

impl<'a> Formatter for OpenStructures<'a> {


    /// Called before every array.  Writes a `[` to the specified
    /// writer.
    #[inline]
    fn begin_array<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.open += 1;
        writer.write_all(b"[")
    }

    /// Called after every array.  Writes a `]` to the specified
    /// writer.
    #[inline]
    fn end_array<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        if self.is_open() {
            self.print_indents_below(writer)?;
        }
        writer.write_all(b"]")?;
        self.open -= 1;
        Ok(())
    }

    /// Called before every array value.  Writes a `,` if needed to
    /// the specified writer.
    #[inline]
    fn begin_array_value<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        if self.is_open() {
            if first {
                writer.write(b"\n")?;
            }
            self.print_indents(writer)?;
        }
        if first || self.is_open() {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    /// Called after every array value.
    #[inline]
    fn end_array_value<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        if self.is_open() {
            writer.write_all(b",\n")
        } else {
            Ok(())
        }

    }

    /// Called before every object.  Writes a `{` to the specified
    /// writer.
    #[inline]
    fn begin_object<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        self.open += 1;
        writer.write_all(b"{")
    }

    /// Called after every object.  Writes a `}` to the specified
    /// writer.
    #[inline]
    fn end_object<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        if self.is_open() {
            self.print_indents_below(writer)?;
        }
        writer.write_all(b"}")?;
        self.open -= 1;
        Ok(())
    }

    /// Called before every object key.
    #[inline]
    fn begin_object_key<W>(&mut self, writer: &mut W, first: bool) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        if self.is_open() {
            if first {
                writer.write(b"\n")?;
            }
            self.print_indents(writer)?;
        }
        if first || self.is_open() {
            Ok(())
        } else {
            writer.write_all(b", ")
        }
    }

    #[inline]
    fn begin_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        writer.write_all(b": ")
    }

    /// Called after every object value.
    #[inline]
    fn end_object_value<W>(&mut self, writer: &mut W) -> io::Result<()>
        where
            W: ?Sized + io::Write,
    {
        if self.is_open() {
            writer.write_all(b",\n")
        } else {
            Ok(())
        }
    }
}