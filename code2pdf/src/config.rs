use std::str::FromStr;

#[derive(Debug)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Syntax {
    Ada,
    Asm,
    Awk,
    Bash,
    C,
    Changelog,
    Cpp,
    Csh,
    Delphi,
    Diff,
    Diffs,
    Diffu,
    Dylan,
    Eiffel,
    Elisp,
    Erlang,
    F90,
    Forth,
    Fortran,
    FortranPp,
    Haskell,
    Html,
    Icon,
    Idl,
    Inf,
    Java,
    JavaScript,
    Ksh,
    Lua,
    M4,
    Mail,
    Makefile,
    Matlab,
    Nroff,
    Oberon2,
    Objc,
    Octave,
    Outline,
    Oz,
    Pascal,
    Perl,
    Postscript,
    Pyrex,
    Python,
    Rfc,
    Ruby,
    Scheme,
    Sh,
    Skill,
    Smalltalk,
    Sml,
    Sql,
    States,
    Synopsys,
    Tcl,
    Tcsh,
    Tex,
    Vba,
    Verilog,
    Vhdl,
    Vrml,
    Wmlscript,
    Zsh,
}

impl Syntax {
    pub fn as_str(&self) -> &'static str {
        match self {
            Syntax::Ada => "ada",
            Syntax::Asm => "asm",
            Syntax::Awk => "awk",
            Syntax::Bash => "bash",
            Syntax::C => "c",
            Syntax::Changelog => "changelog",
            Syntax::Cpp => "cpp",
            Syntax::Csh => "csh",
            Syntax::Delphi => "delphi",
            Syntax::Diff => "diff",
            Syntax::Diffs => "diffs",
            Syntax::Diffu => "diffu",
            Syntax::Dylan => "dylan",
            Syntax::Eiffel => "eiffel",
            Syntax::Elisp => "elisp",
            Syntax::Erlang => "erlang",
            Syntax::F90 => "f90",
            Syntax::Forth => "forth",
            Syntax::Fortran => "fortran",
            Syntax::FortranPp => "fortran_pp",
            Syntax::Haskell => "haskell",
            Syntax::Html => "html",
            Syntax::Icon => "icon",
            Syntax::Idl => "idl",
            Syntax::Inf => "inf",
            Syntax::Java => "java",
            Syntax::JavaScript => "javascript",
            Syntax::Ksh => "ksh",
            Syntax::Lua => "lua",
            Syntax::M4 => "m4",
            Syntax::Mail => "mail",
            Syntax::Makefile => "makefile",
            Syntax::Matlab => "matlab",
            Syntax::Nroff => "nroff",
            Syntax::Oberon2 => "oberon2",
            Syntax::Objc => "objc",
            Syntax::Octave => "octave",
            Syntax::Outline => "outline",
            Syntax::Oz => "oz",
            Syntax::Pascal => "pascal",
            Syntax::Perl => "perl",
            Syntax::Postscript => "postscript",
            Syntax::Pyrex => "pyrex",
            Syntax::Python => "python",
            Syntax::Rfc => "rfc",
            Syntax::Ruby => "ruby",
            Syntax::Scheme => "scheme",
            Syntax::Sh => "sh",
            Syntax::Skill => "skill",
            Syntax::Smalltalk => "Smalltalk",
            Syntax::Sml => "sml",
            Syntax::Sql => "sql",
            Syntax::States => "states",
            Syntax::Synopsys => "synopsys",
            Syntax::Tcl => "tcl",
            Syntax::Tcsh => "tcsh",
            Syntax::Tex => "tex",
            Syntax::Vba => "vba",
            Syntax::Verilog => "verilog",
            Syntax::Vhdl => "vhdl",
            Syntax::Vrml => "vrml",
            Syntax::Wmlscript => "wmlscript",
            Syntax::Zsh => "zsh",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Syntax::Ada => "Ada95",
            Syntax::Asm => "Assembler",
            Syntax::Awk => "AWK",
            Syntax::Bash => "Bourne-Again Shell",
            Syntax::C => "C",
            Syntax::Changelog => "ChangeLog",
            Syntax::Cpp => "C++",
            Syntax::Csh => "C-Shell",
            Syntax::Delphi => "Delphi",
            Syntax::Diff => "Normal Diff",
            Syntax::Diffs => "Side Diff",
            Syntax::Diffu => "Unified Diff",
            Syntax::Dylan => "Dylan",
            Syntax::Eiffel => "Eiffel",
            Syntax::Elisp => "Emacs Lisp",
            Syntax::Erlang => "Erlang",
            Syntax::F90 => "Fortran90",
            Syntax::Forth => "Forth",
            Syntax::Fortran => "Fortran77",
            Syntax::FortranPp => "Fortran77 with CPP",
            Syntax::Haskell => "Haskell",
            Syntax::Html => "HTML",
            Syntax::Icon => "Icon",
            Syntax::Idl => "IDL (CORBA)",
            Syntax::Inf => "INF Script",
            Syntax::Java => "Java",
            Syntax::JavaScript => "JavaScript",
            Syntax::Ksh => "Korn Shell",
            Syntax::Lua => "Lua",
            Syntax::M4 => "M4 Macro",
            Syntax::Mail => "Mail/News",
            Syntax::Makefile => "Makefile",
            Syntax::Matlab => "Matlab",
            Syntax::Nroff => "Nroff",
            Syntax::Oberon2 => "Oberon 2",
            Syntax::Objc => "Objective-C",
            Syntax::Octave => "Octave",
            Syntax::Outline => "Outline",
            Syntax::Oz => "Mozart/Oz",
            Syntax::Pascal => "Pascal",
            Syntax::Perl => "Perl",
            Syntax::Postscript => "PostScript",
            Syntax::Pyrex => "Pyrex",
            Syntax::Python => "Python",
            Syntax::Rfc => "RFC/Internet Draft",
            Syntax::Ruby => "Ruby",
            Syntax::Scheme => "Scheme",
            Syntax::Sh => "Bourne Shell",
            Syntax::Skill => "Skill",
            Syntax::Smalltalk => "Smalltalk",
            Syntax::Sml => "Standard ML",
            Syntax::Sql => "SQL (Sybase 11)",
            Syntax::States => "States",
            Syntax::Synopsys => "Synopsys DC Shell",
            Syntax::Tcl => "Tcl",
            Syntax::Tcsh => "TC-Shell",
            Syntax::Tex => "TeX/LaTeX",
            Syntax::Vba => "Visual Basic",
            Syntax::Verilog => "Verilog",
            Syntax::Vhdl => "VHDL",
            Syntax::Vrml => "VRML",
            Syntax::Wmlscript => "WMLScript",
            Syntax::Zsh => "Z-Shell",
        }
    }
}

impl FromStr for Syntax {
    type Err = Code2PdfError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "ada" => Ok(Syntax::Ada),
            "asm" => Ok(Syntax::Asm),
            "awk" => Ok(Syntax::Awk),
            "bash" => Ok(Syntax::Bash),
            "c" => Ok(Syntax::C),
            "changelog" => Ok(Syntax::Changelog),
            "cpp" => Ok(Syntax::Cpp),
            "csh" => Ok(Syntax::Csh),
            "delphi" => Ok(Syntax::Delphi),
            "diff" => Ok(Syntax::Diff),
            "diffs" => Ok(Syntax::Diffs),
            "diffu" => Ok(Syntax::Diffu),
            "dylan" => Ok(Syntax::Dylan),
            "eiffel" => Ok(Syntax::Eiffel),
            "elisp" => Ok(Syntax::Elisp),
            "erlang" => Ok(Syntax::Erlang),
            "f90" => Ok(Syntax::F90),
            "forth" => Ok(Syntax::Forth),
            "fortran" => Ok(Syntax::Fortran),
            "fortran_pp" => Ok(Syntax::FortranPp),
            "haskell" => Ok(Syntax::Haskell),
            "html" => Ok(Syntax::Html),
            "icon" => Ok(Syntax::Icon),
            "idl" => Ok(Syntax::Idl),
            "inf" => Ok(Syntax::Inf),
            "java" => Ok(Syntax::Java),
            "javascript" => Ok(Syntax::JavaScript),
            "ksh" => Ok(Syntax::Ksh),
            "lua" => Ok(Syntax::Lua),
            "m4" => Ok(Syntax::M4),
            "mail" => Ok(Syntax::Mail),
            "makefile" => Ok(Syntax::Makefile),
            "matlab" => Ok(Syntax::Matlab),
            "nroff" => Ok(Syntax::Nroff),
            "oberon2" => Ok(Syntax::Oberon2),
            "objc" => Ok(Syntax::Objc),
            "octave" => Ok(Syntax::Octave),
            "outline" => Ok(Syntax::Outline),
            "oz" => Ok(Syntax::Oz),
            "pascal" => Ok(Syntax::Pascal),
            "perl" => Ok(Syntax::Perl),
            "postscript" => Ok(Syntax::Postscript),
            "pyrex" => Ok(Syntax::Pyrex),
            "python" => Ok(Syntax::Python),
            "rfc" => Ok(Syntax::Rfc),
            "ruby" => Ok(Syntax::Ruby),
            "scheme" => Ok(Syntax::Scheme),
            "sh" => Ok(Syntax::Sh),
            "skill" => Ok(Syntax::Skill),
            "Smalltalk" => Ok(Syntax::Smalltalk),
            "sml" => Ok(Syntax::Sml),
            "sql" => Ok(Syntax::Sql),
            "states" => Ok(Syntax::States),
            "synopsys" => Ok(Syntax::Synopsys),
            "tcl" => Ok(Syntax::Tcl),
            "tcsh" => Ok(Syntax::Tcsh),
            "tex" => Ok(Syntax::Tex),
            "vba" => Ok(Syntax::Vba),
            "verilog" => Ok(Syntax::Verilog),
            "vhdl" => Ok(Syntax::Vhdl),
            "vrml" => Ok(Syntax::Vrml),
            "wmlscript" => Ok(Syntax::Wmlscript),
            "zsh" => Ok(Syntax::Zsh),
            //"None" => None,
            _ => Err(Code2PdfError::InvalidParameter(format!("Invalid syntax: {}", s))),
        }
    }
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