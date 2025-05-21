use std::str::FromStr;

#[derive(Debug,PartialEq)]
pub enum Code2PdfError {
    InvalidParameter(String),
}

impl std::error::Error for Code2PdfError {}

impl std::fmt::Display for Code2PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Code2PdfError::InvalidParameter(msg) => write!(f, "{}", msg)
        }
    }
}

#[derive(Debug,PartialEq, Clone, Copy)]
pub enum Orientation {
    Portrait,
    Landscape,
}

impl Orientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::Portrait => "Portrait",
            Orientation::Landscape => "Landscape",
        }
    }
}

impl FromStr for Orientation {
    type Err = Code2PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "portrait" => Ok(Orientation::Portrait),
            "landscape" => Ok(Orientation::Landscape),
            _ => Err(Code2PdfError::InvalidParameter(format!("Invalid orientation: {}", s))),
        }
    }
}

#[derive(Debug,PartialEq, Clone, Copy)]
pub enum Media {
    Letter,
    Legal,
    Tabloid,
    A4,
    B4
}

impl Media {
    // Convert to string for Enscript backend
    pub fn as_str(&self) -> &'static str {
        match self {
            Media::Letter => "Letter",
            Media::Legal => "Legal",
            Media::Tabloid => "Tabloid",
            Media::A4 => "A4",
            Media::B4 => "B4",
        }
    }
}

impl FromStr for Media {
    type Err = Code2PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "letter" => Ok(Media::Letter),
            "legal" => Ok(Media::Legal),
            "tabloid" => Ok(Media::Tabloid),
            "a4" => Ok(Media::A4),
            "b4" => Ok(Media::B4),
            _ => Err(Code2PdfError::InvalidParameter(format!("Invalid media: {}", s))),
        }
    }
}

macro_rules! define_syntax {
    ($($variant:ident => $id:literal, $display:literal),* $(,)?) => {
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum Syntax {
            $($variant),*
        }

        impl Syntax {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Syntax::$variant => $id),*
                }
            }

            pub fn display_name(&self) -> &'static str {
                match self {
                    $(Syntax::$variant => $display),*
                }
            }
        }

        impl FromStr for Syntax {
            type Err = Code2PdfError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim().to_lowercase().as_str() {
                    $($id => Ok(Syntax::$variant)),*,
                    _ => Err(Code2PdfError::InvalidParameter(format!("Invalid syntax: {}", s))),
                }
            }
        }

        pub const SYNTAX_OPTIONS: [(Syntax, &'static str, &'static str); 63] = [
            $( (Syntax::$variant, $id, $display) ),*
        ];
    };
}

define_syntax! {
    Ada => "ada", "Ada95",
    Asm => "asm", "Assembler",
    Awk => "awk", "AWK",
    Bash => "bash", "Bourne-Again Shell",
    C => "c", "C",
    Changelog => "changelog", "ChangeLog",
    Cpp => "cpp", "C++",
    Csh => "csh", "C-Shell",
    Delphi => "delphi", "Delphi",
    Diff => "diff", "Normal Diff",
    Diffs => "diffs", "Side Diff",
    Diffu => "diffu", "Unified Diff",
    Dylan => "dylan", "Dylan",
    Eiffel => "eiffel", "Eiffel",
    Elisp => "elisp", "Emacs Lisp",
    Erlang => "erlang", "Erlang",
    F90 => "f90", "Fortran90",
    Forth => "forth", "Forth",
    Fortran => "fortran", "Fortran77",
    FortranPp => "fortran_pp", "Fortran77 with CPP",
    Haskell => "haskell", "Haskell",
    Html => "html", "HTML",
    Icon => "icon", "Icon",
    Idl => "idl", "IDL (CORBA)",
    Inf => "inf", "INF Script",
    Java => "java", "Java",
    JavaScript => "javascript", "JavaScript",
    Ksh => "ksh", "Korn Shell",
    Lua => "lua", "Lua",
    M4 => "m4", "M4 Macro",
    Mail => "mail", "Mail/News",
    Makefile => "makefile", "Makefile",
    Matlab => "matlab", "Matlab",
    Nroff => "nroff", "Nroff",
    Oberon2 => "oberon2", "Oberon 2",
    Objc => "objc", "Objective-C",
    Octave => "octave", "Octave",
    Outline => "outline", "Outline",
    Oz => "oz", "Mozart/Oz",
    Pascal => "pascal", "Pascal",
    Perl => "perl", "Perl",
    Postscript => "postscript", "PostScript",
    Pyrex => "pyrex", "Pyrex",
    Python => "python", "Python",
    Rfc => "rfc", "RFC/Internet Draft",
    Ruby => "ruby", "Ruby",
    Scheme => "scheme", "Scheme",
    Sh => "sh", "Bourne Shell",
    Skill => "skill", "Skill",
    Smalltalk => "smalltalk", "Smalltalk",
    Sml => "sml", "Standard ML",
    Sql => "sql", "SQL (Sybase 11)",
    States => "states", "States",
    Synopsys => "synopsys", "Synopsys DC Shell",
    Tcl => "tcl", "Tcl",
    Tcsh => "tcsh", "TC-Shell",
    Tex => "tex", "TeX/LaTeX",
    Vba => "vba", "Visual Basic",
    Verilog => "verilog", "Verilog",
    Vhdl => "vhdl", "VHDL",
    Vrml => "vrml", "VRML",
    Wmlscript => "wmlscript", "WMLScript",
    Zsh => "zsh", "Z-Shell"
}

#[derive(Debug,PartialEq)]
// This struct and all members are `pub` because it represents the closest thing to a stable
// public interface this program/library will ever have.
pub struct EnscriptContext<'a> {
    // Title of output PDF document
    pub title: Option<String>,
    // Media type
    pub media: Media,
    // Language to use for syntax highlighting
    pub syntax: Option<Syntax>,
    pub font: String,
    pub color: bool,
    pub tabsize: u32,
    // Number of columns in output PDF
    pub columns: u32,
    // Page orientation for output PDF
    pub orientation: Orientation,
    // Code buffer
    pub code: &'a str,
}
// PDF viewer program should be saved, but doesn't need to be shared with the backend

impl EnscriptContext<'_> {
    // Getter for title
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    // Getter for media
    pub fn media(&self) -> Media {
        self.media
    }

    // Getter for syntax
    pub fn syntax(&self) -> Option<Syntax> {
        self.syntax
    }

    // Getter for font
    pub fn font(&self) -> &str {
        &self.font
    }

    // Getter for color
    pub fn color(&self) -> bool {
        self.color
    }

    // Getter for tabsize
    pub fn tabsize(&self) -> u32 {
        self.tabsize
    }

    // Getter for columns
    pub fn columns(&self) -> u32 {
        self.columns
    }

    // Getter for orientation
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    // Getter for code
    pub fn code(&self) -> &str {
        &self.code
    }
}

impl Default for EnscriptContext<'_> {
    fn default() -> Self {
        EnscriptContext {
            title: None,
            media: Media::Letter,
            syntax: None,
            font: "Courier7".into(),
            color: true,
            tabsize: 4,
            columns: 2,
            orientation: Orientation::Landscape,
            code: "",
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_from_str() {
        assert!("bash".parse::<Syntax>().unwrap() == Syntax::Bash);
        assert!("Roentgenium".parse::<Syntax>().is_err());
    }

    #[test]
    fn test_syntax_as_str() {
        assert!(Syntax::JavaScript.as_str() == "javascript");
    }

    #[test]
    fn test_syntax_display_name() {
        assert!(Syntax::Html.display_name() == "HTML");
    }

    #[test]
    fn test_syntax_options() {
        assert!(SYNTAX_OPTIONS.iter().any(|&x| {
            x.1 == "cpp" && x.2 == "C++"
        }));
    }
}
