use std::collections::LinkedList;
use std::collections::HashMap;
use std::borrow::Borrow;

use token::Token;

use std::cell::Cell;
use std::rc::Rc;
use regex::Regex;
use context::Context;
use token::Posn;
use token::ReservedWord;

macro_rules! match_one_char_class {
    ( ( $start:expr, $end:expr ) ) => {
        $start ... $end
    };
    ( $single:expr ) => {
        $single
    }
}

macro_rules! match_char_class {
    ( $x:expr, [ $( $c:tt ),* ] ) => {
        match $x {
            $(
                match_one_char_class!($c)
            )|* => true,
            _ => false
        }
    }
}

pub trait ESCharExt {
    fn is_es_newline(self) -> bool;
    fn is_es_whitespace(self) -> bool;
    fn is_es_identifier(self) -> bool;
    fn is_es_identifier_start(self) -> bool;
    fn is_es_identifier_continue(self) -> bool;
    fn is_es_single_escape_char(self) -> bool;
    fn is_es_hex_digit(self) -> bool;
    fn is_es_oct_digit(self) -> bool;
    fn is_es_nonascii_identifier_start(self) -> bool;
    fn is_es_nonascii_identifier_part(self) -> bool;
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    UnexpectedEOF,
    // FIXME: split this up into specific situational errors
    UnexpectedChar(char),
    InvalidDigit(char)
}

/*
// https://github.com/ariya/esprima/blob/master/tools/generate-identifier-regex.js
// https://gist.github.com/mathiasbynens/6334847
// http://unicode.org/reports/tr31/

pub trait UnicodeCharExt {
    fn is_L(self) -> bool;
    fn is_Lu(self) -> bool;
    fn is_Ll(self) -> bool;
    fn is_Lt(self) -> bool;
    fn is_Lm(self) -> bool;
    fn is_Lo(self) -> bool;
    fn is_Nl(self) -> bool;
    fn is_OtherID_Start(self) -> bool;
}

impl UnicodeCharExt for char {
    fn is_L(self) -> bool {
        self.is_Lu() ||
        self.is_Ll() ||
        self.is_Lt() ||
        self.is_Lm() ||
        self.is_Lo()
    }

    fn is_Lu(self) -> bool {
        false
    }

    fn is_Ll(self) -> bool {
        false
    }

    fn is_Lt(self) -> bool {
        false
    }

    fn is_Lm(self) -> bool {
        false
    }

    fn is_Lo(self) -> bool {
        false
    }

    fn is_Nl(self) -> bool {
        false
    }

    fn is_OtherID_Start(self) -> bool {
        false
    }
}
*/

impl ESCharExt for char {
    fn is_es_newline(self) -> bool {
        match self {
            '\u{000a}' | '\u{000d}' | '\u{2028}' | '\u{2029}' => true,
            _ => false
        }
    }

    fn is_es_whitespace(self) -> bool {
        match self {
              '\u{0009}' | '\u{000b}' | '\u{000c}' | '\u{0020}' | '\u{00a0}'
            | '\u{1680}' | '\u{2000}' | '\u{2001}' | '\u{2002}' | '\u{2003}' | '\u{2004}'
            | '\u{2005}' | '\u{2006}' | '\u{2009}' | '\u{200a}' | '\u{202f}' | '\u{205f}'
            | '\u{3000}' | '\u{feff}' => true,
            _ => false
        }
    }

    fn is_es_identifier(self) -> bool {
        self.is_es_identifier_continue()
    }

    fn is_es_identifier_start(self) -> bool {
        self == '$' ||
        self == '_' ||
        self.is_alphabetic()
    }

    fn is_es_identifier_continue(self) -> bool {
        self.is_es_identifier_start() ||
        self.is_numeric()
    }

    fn is_es_single_escape_char(self) -> bool {
        match self {
            '\'' | '"' | '\\' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' => true,
            _ => false
        }
    }

    fn is_es_hex_digit(self) -> bool {
        match self {
              '0'...'9'
            | 'a'...'f'
            | 'A'...'F' => true,
            _ => false
        }
    }

    fn is_es_oct_digit(self) -> bool {
        match self {
            '0'...'7' => true,
            _ => false
        }
    }

    fn is_es_nonascii_identifier_start(self) -> bool {
        match_char_class!(
            self,
            ['\u{00AA}','\u{00B5}','\u{00BA}',('\u{00C0}','\u{00D6}'),('\u{00D8}','\u{00F6}'),('\u{00F8}','\u{02C1}'),('\u{02C6}','\u{02D1}'),('\u{02E0}','\u{02E4}'),'\u{02EC}','\u{02EE}',('\u{0370}','\u{0374}'),'\u{0376}','\u{0377}',('\u{037A}','\u{037D}'),'\u{037F}','\u{0386}',('\u{0388}','\u{038A}'),'\u{038C}',('\u{038E}','\u{03A1}'),('\u{03A3}','\u{03F5}'),('\u{03F7}','\u{0481}'),('\u{048A}','\u{052F}'),('\u{0531}','\u{0556}'),'\u{0559}',('\u{0561}','\u{0587}'),('\u{05D0}','\u{05EA}'),('\u{05F0}','\u{05F2}'),('\u{0620}','\u{064A}'),'\u{066E}','\u{066F}',('\u{0671}','\u{06D3}'),'\u{06D5}','\u{06E5}','\u{06E6}','\u{06EE}','\u{06EF}',('\u{06FA}','\u{06FC}'),'\u{06FF}','\u{0710}',('\u{0712}','\u{072F}'),('\u{074D}','\u{07A5}'),'\u{07B1}',('\u{07CA}','\u{07EA}'),'\u{07F4}','\u{07F5}','\u{07FA}',('\u{0800}','\u{0815}'),'\u{081A}','\u{0824}','\u{0828}',('\u{0840}','\u{0858}'),('\u{08A0}','\u{08B2}'),('\u{0904}','\u{0939}'),'\u{093D}','\u{0950}',('\u{0958}','\u{0961}'),('\u{0971}','\u{0980}'),('\u{0985}','\u{098C}'),'\u{098F}','\u{0990}',('\u{0993}','\u{09A8}'),('\u{09AA}','\u{09B0}'),'\u{09B2}',('\u{09B6}','\u{09B9}'),'\u{09BD}','\u{09CE}','\u{09DC}','\u{09DD}',('\u{09DF}','\u{09E1}'),'\u{09F0}','\u{09F1}',('\u{0A05}','\u{0A0A}'),'\u{0A0F}','\u{0A10}',('\u{0A13}','\u{0A28}'),('\u{0A2A}','\u{0A30}'),'\u{0A32}','\u{0A33}','\u{0A35}','\u{0A36}','\u{0A38}','\u{0A39}',('\u{0A59}','\u{0A5C}'),'\u{0A5E}',('\u{0A72}','\u{0A74}'),('\u{0A85}','\u{0A8D}'),('\u{0A8F}','\u{0A91}'),('\u{0A93}','\u{0AA8}'),('\u{0AAA}','\u{0AB0}'),'\u{0AB2}','\u{0AB3}',('\u{0AB5}','\u{0AB9}'),'\u{0ABD}','\u{0AD0}','\u{0AE0}','\u{0AE1}',('\u{0B05}','\u{0B0C}'),'\u{0B0F}','\u{0B10}',('\u{0B13}','\u{0B28}'),('\u{0B2A}','\u{0B30}'),'\u{0B32}','\u{0B33}',('\u{0B35}','\u{0B39}'),'\u{0B3D}','\u{0B5C}','\u{0B5D}',('\u{0B5F}','\u{0B61}'),'\u{0B71}','\u{0B83}',('\u{0B85}','\u{0B8A}'),('\u{0B8E}','\u{0B90}'),('\u{0B92}','\u{0B95}'),'\u{0B99}','\u{0B9A}','\u{0B9C}','\u{0B9E}','\u{0B9F}','\u{0BA3}','\u{0BA4}',('\u{0BA8}','\u{0BAA}'),('\u{0BAE}','\u{0BB9}'),'\u{0BD0}',('\u{0C05}','\u{0C0C}'),('\u{0C0E}','\u{0C10}'),('\u{0C12}','\u{0C28}'),('\u{0C2A}','\u{0C39}'),'\u{0C3D}','\u{0C58}','\u{0C59}','\u{0C60}','\u{0C61}',('\u{0C85}','\u{0C8C}'),('\u{0C8E}','\u{0C90}'),('\u{0C92}','\u{0CA8}'),('\u{0CAA}','\u{0CB3}'),('\u{0CB5}','\u{0CB9}'),'\u{0CBD}','\u{0CDE}','\u{0CE0}','\u{0CE1}','\u{0CF1}','\u{0CF2}',('\u{0D05}','\u{0D0C}'),('\u{0D0E}','\u{0D10}'),('\u{0D12}','\u{0D3A}'),'\u{0D3D}','\u{0D4E}','\u{0D60}','\u{0D61}',('\u{0D7A}','\u{0D7F}'),('\u{0D85}','\u{0D96}'),('\u{0D9A}','\u{0DB1}'),('\u{0DB3}','\u{0DBB}'),'\u{0DBD}',('\u{0DC0}','\u{0DC6}'),('\u{0E01}','\u{0E30}'),'\u{0E32}','\u{0E33}',('\u{0E40}','\u{0E46}'),'\u{0E81}','\u{0E82}','\u{0E84}','\u{0E87}','\u{0E88}','\u{0E8A}','\u{0E8D}',('\u{0E94}','\u{0E97}'),('\u{0E99}','\u{0E9F}'),('\u{0EA1}','\u{0EA3}'),'\u{0EA5}','\u{0EA7}','\u{0EAA}','\u{0EAB}',('\u{0EAD}','\u{0EB0}'),'\u{0EB2}','\u{0EB3}','\u{0EBD}',('\u{0EC0}','\u{0EC4}'),'\u{0EC6}',('\u{0EDC}','\u{0EDF}'),'\u{0F00}',('\u{0F40}','\u{0F47}'),('\u{0F49}','\u{0F6C}'),('\u{0F88}','\u{0F8C}'),('\u{1000}','\u{102A}'),'\u{103F}',('\u{1050}','\u{1055}'),('\u{105A}','\u{105D}'),'\u{1061}','\u{1065}','\u{1066}',('\u{106E}','\u{1070}'),('\u{1075}','\u{1081}'),'\u{108E}',('\u{10A0}','\u{10C5}'),'\u{10C7}','\u{10CD}',('\u{10D0}','\u{10FA}'),('\u{10FC}','\u{1248}'),('\u{124A}','\u{124D}'),('\u{1250}','\u{1256}'),'\u{1258}',('\u{125A}','\u{125D}'),('\u{1260}','\u{1288}'),('\u{128A}','\u{128D}'),('\u{1290}','\u{12B0}'),('\u{12B2}','\u{12B5}'),('\u{12B8}','\u{12BE}'),'\u{12C0}',('\u{12C2}','\u{12C5}'),('\u{12C8}','\u{12D6}'),('\u{12D8}','\u{1310}'),('\u{1312}','\u{1315}'),('\u{1318}','\u{135A}'),('\u{1380}','\u{138F}'),('\u{13A0}','\u{13F4}'),('\u{1401}','\u{166C}'),('\u{166F}','\u{167F}'),('\u{1681}','\u{169A}'),('\u{16A0}','\u{16EA}'),('\u{16EE}','\u{16F8}'),('\u{1700}','\u{170C}'),('\u{170E}','\u{1711}'),('\u{1720}','\u{1731}'),('\u{1740}','\u{1751}'),('\u{1760}','\u{176C}'),('\u{176E}','\u{1770}'),('\u{1780}','\u{17B3}'),'\u{17D7}','\u{17DC}',('\u{1820}','\u{1877}'),('\u{1880}','\u{18A8}'),'\u{18AA}',('\u{18B0}','\u{18F5}'),('\u{1900}','\u{191E}'),('\u{1950}','\u{196D}'),('\u{1970}','\u{1974}'),('\u{1980}','\u{19AB}'),('\u{19C1}','\u{19C7}'),('\u{1A00}','\u{1A16}'),('\u{1A20}','\u{1A54}'),'\u{1AA7}',('\u{1B05}','\u{1B33}'),('\u{1B45}','\u{1B4B}'),('\u{1B83}','\u{1BA0}'),'\u{1BAE}','\u{1BAF}',('\u{1BBA}','\u{1BE5}'),('\u{1C00}','\u{1C23}'),('\u{1C4D}','\u{1C4F}'),('\u{1C5A}','\u{1C7D}'),('\u{1CE9}','\u{1CEC}'),('\u{1CEE}','\u{1CF1}'),'\u{1CF5}','\u{1CF6}',('\u{1D00}','\u{1DBF}'),('\u{1E00}','\u{1F15}'),('\u{1F18}','\u{1F1D}'),('\u{1F20}','\u{1F45}'),('\u{1F48}','\u{1F4D}'),('\u{1F50}','\u{1F57}'),'\u{1F59}','\u{1F5B}','\u{1F5D}',('\u{1F5F}','\u{1F7D}'),('\u{1F80}','\u{1FB4}'),('\u{1FB6}','\u{1FBC}'),'\u{1FBE}',('\u{1FC2}','\u{1FC4}'),('\u{1FC6}','\u{1FCC}'),('\u{1FD0}','\u{1FD3}'),('\u{1FD6}','\u{1FDB}'),('\u{1FE0}','\u{1FEC}'),('\u{1FF2}','\u{1FF4}'),('\u{1FF6}','\u{1FFC}'),'\u{2071}','\u{207F}',('\u{2090}','\u{209C}'),'\u{2102}','\u{2107}',('\u{210A}','\u{2113}'),'\u{2115}',('\u{2119}','\u{211D}'),'\u{2124}','\u{2126}','\u{2128}',('\u{212A}','\u{212D}'),('\u{212F}','\u{2139}'),('\u{213C}','\u{213F}'),('\u{2145}','\u{2149}'),'\u{214E}',('\u{2160}','\u{2188}'),('\u{2C00}','\u{2C2E}'),('\u{2C30}','\u{2C5E}'),('\u{2C60}','\u{2CE4}'),('\u{2CEB}','\u{2CEE}'),'\u{2CF2}','\u{2CF3}',('\u{2D00}','\u{2D25}'),'\u{2D27}','\u{2D2D}',('\u{2D30}','\u{2D67}'),'\u{2D6F}',('\u{2D80}','\u{2D96}'),('\u{2DA0}','\u{2DA6}'),('\u{2DA8}','\u{2DAE}'),('\u{2DB0}','\u{2DB6}'),('\u{2DB8}','\u{2DBE}'),('\u{2DC0}','\u{2DC6}'),('\u{2DC8}','\u{2DCE}'),('\u{2DD0}','\u{2DD6}'),('\u{2DD8}','\u{2DDE}'),'\u{2E2F}',('\u{3005}','\u{3007}'),('\u{3021}','\u{3029}'),('\u{3031}','\u{3035}'),('\u{3038}','\u{303C}'),('\u{3041}','\u{3096}'),('\u{309D}','\u{309F}'),('\u{30A1}','\u{30FA}'),('\u{30FC}','\u{30FF}'),('\u{3105}','\u{312D}'),('\u{3131}','\u{318E}'),('\u{31A0}','\u{31BA}'),('\u{31F0}','\u{31FF}'),('\u{3400}','\u{4DB5}'),('\u{4E00}','\u{9FCC}'),('\u{A000}','\u{A48C}'),('\u{A4D0}','\u{A4FD}'),('\u{A500}','\u{A60C}'),('\u{A610}','\u{A61F}'),'\u{A62A}','\u{A62B}',('\u{A640}','\u{A66E}'),('\u{A67F}','\u{A69D}'),('\u{A6A0}','\u{A6EF}'),('\u{A717}','\u{A71F}'),('\u{A722}','\u{A788}'),('\u{A78B}','\u{A78E}'),('\u{A790}','\u{A7AD}'),'\u{A7B0}','\u{A7B1}',('\u{A7F7}','\u{A801}'),('\u{A803}','\u{A805}'),('\u{A807}','\u{A80A}'),('\u{A80C}','\u{A822}'),('\u{A840}','\u{A873}'),('\u{A882}','\u{A8B3}'),('\u{A8F2}','\u{A8F7}'),'\u{A8FB}',('\u{A90A}','\u{A925}'),('\u{A930}','\u{A946}'),('\u{A960}','\u{A97C}'),('\u{A984}','\u{A9B2}'),'\u{A9CF}',('\u{A9E0}','\u{A9E4}'),('\u{A9E6}','\u{A9EF}'),('\u{A9FA}','\u{A9FE}'),('\u{AA00}','\u{AA28}'),('\u{AA40}','\u{AA42}'),('\u{AA44}','\u{AA4B}'),('\u{AA60}','\u{AA76}'),'\u{AA7A}',('\u{AA7E}','\u{AAAF}'),'\u{AAB1}','\u{AAB5}','\u{AAB6}',('\u{AAB9}','\u{AABD}'),'\u{AAC0}','\u{AAC2}',('\u{AADB}','\u{AADD}'),('\u{AAE0}','\u{AAEA}'),('\u{AAF2}','\u{AAF4}'),('\u{AB01}','\u{AB06}'),('\u{AB09}','\u{AB0E}'),('\u{AB11}','\u{AB16}'),('\u{AB20}','\u{AB26}'),('\u{AB28}','\u{AB2E}'),('\u{AB30}','\u{AB5A}'),('\u{AB5C}','\u{AB5F}'),'\u{AB64}','\u{AB65}',('\u{ABC0}','\u{ABE2}'),('\u{AC00}','\u{D7A3}'),('\u{D7B0}','\u{D7C6}'),('\u{D7CB}','\u{D7FB}'),('\u{F900}','\u{FA6D}'),('\u{FA70}','\u{FAD9}'),('\u{FB00}','\u{FB06}'),('\u{FB13}','\u{FB17}'),'\u{FB1D}',('\u{FB1F}','\u{FB28}'),('\u{FB2A}','\u{FB36}'),('\u{FB38}','\u{FB3C}'),'\u{FB3E}','\u{FB40}','\u{FB41}','\u{FB43}','\u{FB44}',('\u{FB46}','\u{FBB1}'),('\u{FBD3}','\u{FD3D}'),('\u{FD50}','\u{FD8F}'),('\u{FD92}','\u{FDC7}'),('\u{FDF0}','\u{FDFB}'),('\u{FE70}','\u{FE74}'),('\u{FE76}','\u{FEFC}'),('\u{FF21}','\u{FF3A}'),('\u{FF41}','\u{FF5A}'),('\u{FF66}','\u{FFBE}'),('\u{FFC2}','\u{FFC7}'),('\u{FFCA}','\u{FFCF}'),('\u{FFD2}','\u{FFD7}'),('\u{FFDA}','\u{FFDC}')])
    }

    fn is_es_nonascii_identifier_part(self) -> bool {
        match_char_class!(
            self,
            ['\u{00AA}','\u{00B5}','\u{00BA}',('\u{00C0}','\u{00D6}'),('\u{00D8}','\u{00F6}'),('\u{00F8}','\u{02C1}'),('\u{02C6}','\u{02D1}'),('\u{02E0}','\u{02E4}'),'\u{02EC}','\u{02EE}',('\u{0300}','\u{0374}'),'\u{0376}','\u{0377}',('\u{037A}','\u{037D}'),'\u{037F}','\u{0386}',('\u{0388}','\u{038A}'),'\u{038C}',('\u{038E}','\u{03A1}'),('\u{03A3}','\u{03F5}'),('\u{03F7}','\u{0481}'),('\u{0483}','\u{0487}'),('\u{048A}','\u{052F}'),('\u{0531}','\u{0556}'),'\u{0559}',('\u{0561}','\u{0587}'),('\u{0591}','\u{05BD}'),'\u{05BF}','\u{05C1}','\u{05C2}','\u{05C4}','\u{05C5}','\u{05C7}',('\u{05D0}','\u{05EA}'),('\u{05F0}','\u{05F2}'),('\u{0610}','\u{061A}'),('\u{0620}','\u{0669}'),('\u{066E}','\u{06D3}'),('\u{06D5}','\u{06DC}'),('\u{06DF}','\u{06E8}'),('\u{06EA}','\u{06FC}'),'\u{06FF}',('\u{0710}','\u{074A}'),('\u{074D}','\u{07B1}'),('\u{07C0}','\u{07F5}'),'\u{07FA}',('\u{0800}','\u{082D}'),('\u{0840}','\u{085B}'),('\u{08A0}','\u{08B2}'),('\u{08E4}','\u{0963}'),('\u{0966}','\u{096F}'),('\u{0971}','\u{0983}'),('\u{0985}','\u{098C}'),'\u{098F}','\u{0990}',('\u{0993}','\u{09A8}'),('\u{09AA}','\u{09B0}'),'\u{09B2}',('\u{09B6}','\u{09B9}'),('\u{09BC}','\u{09C4}'),'\u{09C7}','\u{09C8}',('\u{09CB}','\u{09CE}'),'\u{09D7}','\u{09DC}','\u{09DD}',('\u{09DF}','\u{09E3}'),('\u{09E6}','\u{09F1}'),('\u{0A01}','\u{0A03}'),('\u{0A05}','\u{0A0A}'),'\u{0A0F}','\u{0A10}',('\u{0A13}','\u{0A28}'),('\u{0A2A}','\u{0A30}'),'\u{0A32}','\u{0A33}','\u{0A35}','\u{0A36}','\u{0A38}','\u{0A39}','\u{0A3C}',('\u{0A3E}','\u{0A42}'),'\u{0A47}','\u{0A48}',('\u{0A4B}','\u{0A4D}'),'\u{0A51}',('\u{0A59}','\u{0A5C}'),'\u{0A5E}',('\u{0A66}','\u{0A75}'),('\u{0A81}','\u{0A83}'),('\u{0A85}','\u{0A8D}'),('\u{0A8F}','\u{0A91}'),('\u{0A93}','\u{0AA8}'),('\u{0AAA}','\u{0AB0}'),'\u{0AB2}','\u{0AB3}',('\u{0AB5}','\u{0AB9}'),('\u{0ABC}','\u{0AC5}'),('\u{0AC7}','\u{0AC9}'),('\u{0ACB}','\u{0ACD}'),'\u{0AD0}',('\u{0AE0}','\u{0AE3}'),('\u{0AE6}','\u{0AEF}'),('\u{0B01}','\u{0B03}'),('\u{0B05}','\u{0B0C}'),'\u{0B0F}','\u{0B10}',('\u{0B13}','\u{0B28}'),('\u{0B2A}','\u{0B30}'),'\u{0B32}','\u{0B33}',('\u{0B35}','\u{0B39}'),('\u{0B3C}','\u{0B44}'),'\u{0B47}','\u{0B48}',('\u{0B4B}','\u{0B4D}'),'\u{0B56}','\u{0B57}','\u{0B5C}','\u{0B5D}',('\u{0B5F}','\u{0B63}'),('\u{0B66}','\u{0B6F}'),'\u{0B71}','\u{0B82}','\u{0B83}',('\u{0B85}','\u{0B8A}'),('\u{0B8E}','\u{0B90}'),('\u{0B92}','\u{0B95}'),'\u{0B99}','\u{0B9A}','\u{0B9C}','\u{0B9E}','\u{0B9F}','\u{0BA3}','\u{0BA4}',('\u{0BA8}','\u{0BAA}'),('\u{0BAE}','\u{0BB9}'),('\u{0BBE}','\u{0BC2}'),('\u{0BC6}','\u{0BC8}'),('\u{0BCA}','\u{0BCD}'),'\u{0BD0}','\u{0BD7}',('\u{0BE6}','\u{0BEF}'),('\u{0C00}','\u{0C03}'),('\u{0C05}','\u{0C0C}'),('\u{0C0E}','\u{0C10}'),('\u{0C12}','\u{0C28}'),('\u{0C2A}','\u{0C39}'),('\u{0C3D}','\u{0C44}'),('\u{0C46}','\u{0C48}'),('\u{0C4A}','\u{0C4D}'),'\u{0C55}','\u{0C56}','\u{0C58}','\u{0C59}',('\u{0C60}','\u{0C63}'),('\u{0C66}','\u{0C6F}'),('\u{0C81}','\u{0C83}'),('\u{0C85}','\u{0C8C}'),('\u{0C8E}','\u{0C90}'),('\u{0C92}','\u{0CA8}'),('\u{0CAA}','\u{0CB3}'),('\u{0CB5}','\u{0CB9}'),('\u{0CBC}','\u{0CC4}'),('\u{0CC6}','\u{0CC8}'),('\u{0CCA}','\u{0CCD}'),'\u{0CD5}','\u{0CD6}','\u{0CDE}',('\u{0CE0}','\u{0CE3}'),('\u{0CE6}','\u{0CEF}'),'\u{0CF1}','\u{0CF2}',('\u{0D01}','\u{0D03}'),('\u{0D05}','\u{0D0C}'),('\u{0D0E}','\u{0D10}'),('\u{0D12}','\u{0D3A}'),('\u{0D3D}','\u{0D44}'),('\u{0D46}','\u{0D48}'),('\u{0D4A}','\u{0D4E}'),'\u{0D57}',('\u{0D60}','\u{0D63}'),('\u{0D66}','\u{0D6F}'),('\u{0D7A}','\u{0D7F}'),'\u{0D82}','\u{0D83}',('\u{0D85}','\u{0D96}'),('\u{0D9A}','\u{0DB1}'),('\u{0DB3}','\u{0DBB}'),'\u{0DBD}',('\u{0DC0}','\u{0DC6}'),'\u{0DCA}',('\u{0DCF}','\u{0DD4}'),'\u{0DD6}',('\u{0DD8}','\u{0DDF}'),('\u{0DE6}','\u{0DEF}'),'\u{0DF2}','\u{0DF3}',('\u{0E01}','\u{0E3A}'),('\u{0E40}','\u{0E4E}'),('\u{0E50}','\u{0E59}'),'\u{0E81}','\u{0E82}','\u{0E84}','\u{0E87}','\u{0E88}','\u{0E8A}','\u{0E8D}',('\u{0E94}','\u{0E97}'),('\u{0E99}','\u{0E9F}'),('\u{0EA1}','\u{0EA3}'),'\u{0EA5}','\u{0EA7}','\u{0EAA}','\u{0EAB}',('\u{0EAD}','\u{0EB9}'),('\u{0EBB}','\u{0EBD}'),('\u{0EC0}','\u{0EC4}'),'\u{0EC6}',('\u{0EC8}','\u{0ECD}'),('\u{0ED0}','\u{0ED9}'),('\u{0EDC}','\u{0EDF}'),'\u{0F00}','\u{0F18}','\u{0F19}',('\u{0F20}','\u{0F29}'),'\u{0F35}','\u{0F37}','\u{0F39}',('\u{0F3E}','\u{0F47}'),('\u{0F49}','\u{0F6C}'),('\u{0F71}','\u{0F84}'),('\u{0F86}','\u{0F97}'),('\u{0F99}','\u{0FBC}'),'\u{0FC6}',('\u{1000}','\u{1049}'),('\u{1050}','\u{109D}'),('\u{10A0}','\u{10C5}'),'\u{10C7}','\u{10CD}',('\u{10D0}','\u{10FA}'),('\u{10FC}','\u{1248}'),('\u{124A}','\u{124D}'),('\u{1250}','\u{1256}'),'\u{1258}',('\u{125A}','\u{125D}'),('\u{1260}','\u{1288}'),('\u{128A}','\u{128D}'),('\u{1290}','\u{12B0}'),('\u{12B2}','\u{12B5}'),('\u{12B8}','\u{12BE}'),'\u{12C0}',('\u{12C2}','\u{12C5}'),('\u{12C8}','\u{12D6}'),('\u{12D8}','\u{1310}'),('\u{1312}','\u{1315}'),('\u{1318}','\u{135A}'),('\u{135D}','\u{135F}'),('\u{1380}','\u{138F}'),('\u{13A0}','\u{13F4}'),('\u{1401}','\u{166C}'),('\u{166F}','\u{167F}'),('\u{1681}','\u{169A}'),('\u{16A0}','\u{16EA}'),('\u{16EE}','\u{16F8}'),('\u{1700}','\u{170C}'),('\u{170E}','\u{1714}'),('\u{1720}','\u{1734}'),('\u{1740}','\u{1753}'),('\u{1760}','\u{176C}'),('\u{176E}','\u{1770}'),'\u{1772}','\u{1773}',('\u{1780}','\u{17D3}'),'\u{17D7}','\u{17DC}','\u{17DD}',('\u{17E0}','\u{17E9}'),('\u{180B}','\u{180D}'),('\u{1810}','\u{1819}'),('\u{1820}','\u{1877}'),('\u{1880}','\u{18AA}'),('\u{18B0}','\u{18F5}'),('\u{1900}','\u{191E}'),('\u{1920}','\u{192B}'),('\u{1930}','\u{193B}'),('\u{1946}','\u{196D}'),('\u{1970}','\u{1974}'),('\u{1980}','\u{19AB}'),('\u{19B0}','\u{19C9}'),('\u{19D0}','\u{19D9}'),('\u{1A00}','\u{1A1B}'),('\u{1A20}','\u{1A5E}'),('\u{1A60}','\u{1A7C}'),('\u{1A7F}','\u{1A89}'),('\u{1A90}','\u{1A99}'),'\u{1AA7}',('\u{1AB0}','\u{1ABD}'),('\u{1B00}','\u{1B4B}'),('\u{1B50}','\u{1B59}'),('\u{1B6B}','\u{1B73}'),('\u{1B80}','\u{1BF3}'),('\u{1C00}','\u{1C37}'),('\u{1C40}','\u{1C49}'),('\u{1C4D}','\u{1C7D}'),('\u{1CD0}','\u{1CD2}'),('\u{1CD4}','\u{1CF6}'),'\u{1CF8}','\u{1CF9}',('\u{1D00}','\u{1DF5}'),('\u{1DFC}','\u{1F15}'),('\u{1F18}','\u{1F1D}'),('\u{1F20}','\u{1F45}'),('\u{1F48}','\u{1F4D}'),('\u{1F50}','\u{1F57}'),'\u{1F59}','\u{1F5B}','\u{1F5D}',('\u{1F5F}','\u{1F7D}'),('\u{1F80}','\u{1FB4}'),('\u{1FB6}','\u{1FBC}'),'\u{1FBE}',('\u{1FC2}','\u{1FC4}'),('\u{1FC6}','\u{1FCC}'),('\u{1FD0}','\u{1FD3}'),('\u{1FD6}','\u{1FDB}'),('\u{1FE0}','\u{1FEC}'),('\u{1FF2}','\u{1FF4}'),('\u{1FF6}','\u{1FFC}'),'\u{200C}','\u{200D}','\u{203F}','\u{2040}','\u{2054}','\u{2071}','\u{207F}',('\u{2090}','\u{209C}'),('\u{20D0}','\u{20DC}'),'\u{20E1}',('\u{20E5}','\u{20F0}'),'\u{2102}','\u{2107}',('\u{210A}','\u{2113}'),'\u{2115}',('\u{2119}','\u{211D}'),'\u{2124}','\u{2126}','\u{2128}',('\u{212A}','\u{212D}'),('\u{212F}','\u{2139}'),('\u{213C}','\u{213F}'),('\u{2145}','\u{2149}'),'\u{214E}',('\u{2160}','\u{2188}'),('\u{2C00}','\u{2C2E}'),('\u{2C30}','\u{2C5E}'),('\u{2C60}','\u{2CE4}'),('\u{2CEB}','\u{2CF3}'),('\u{2D00}','\u{2D25}'),'\u{2D27}','\u{2D2D}',('\u{2D30}','\u{2D67}'),'\u{2D6F}',('\u{2D7F}','\u{2D96}'),('\u{2DA0}','\u{2DA6}'),('\u{2DA8}','\u{2DAE}'),('\u{2DB0}','\u{2DB6}'),('\u{2DB8}','\u{2DBE}'),('\u{2DC0}','\u{2DC6}'),('\u{2DC8}','\u{2DCE}'),('\u{2DD0}','\u{2DD6}'),('\u{2DD8}','\u{2DDE}'),('\u{2DE0}','\u{2DFF}'),'\u{2E2F}',('\u{3005}','\u{3007}'),('\u{3021}','\u{302F}'),('\u{3031}','\u{3035}'),('\u{3038}','\u{303C}'),('\u{3041}','\u{3096}'),'\u{3099}','\u{309A}',('\u{309D}','\u{309F}'),('\u{30A1}','\u{30FA}'),('\u{30FC}','\u{30FF}'),('\u{3105}','\u{312D}'),('\u{3131}','\u{318E}'),('\u{31A0}','\u{31BA}'),('\u{31F0}','\u{31FF}'),('\u{3400}','\u{4DB5}'),('\u{4E00}','\u{9FCC}'),('\u{A000}','\u{A48C}'),('\u{A4D0}','\u{A4FD}'),('\u{A500}','\u{A60C}'),('\u{A610}','\u{A62B}'),('\u{A640}','\u{A66F}'),('\u{A674}','\u{A67D}'),('\u{A67F}','\u{A69D}'),('\u{A69F}','\u{A6F1}'),('\u{A717}','\u{A71F}'),('\u{A722}','\u{A788}'),('\u{A78B}','\u{A78E}'),('\u{A790}','\u{A7AD}'),'\u{A7B0}','\u{A7B1}',('\u{A7F7}','\u{A827}'),('\u{A840}','\u{A873}'),('\u{A880}','\u{A8C4}'),('\u{A8D0}','\u{A8D9}'),('\u{A8E0}','\u{A8F7}'),'\u{A8FB}',('\u{A900}','\u{A92D}'),('\u{A930}','\u{A953}'),('\u{A960}','\u{A97C}'),('\u{A980}','\u{A9C0}'),('\u{A9CF}','\u{A9D9}'),('\u{A9E0}','\u{A9FE}'),('\u{AA00}','\u{AA36}'),('\u{AA40}','\u{AA4D}'),('\u{AA50}','\u{AA59}'),('\u{AA60}','\u{AA76}'),('\u{AA7A}','\u{AAC2}'),('\u{AADB}','\u{AADD}'),('\u{AAE0}','\u{AAEF}'),('\u{AAF2}','\u{AAF6}'),('\u{AB01}','\u{AB06}'),('\u{AB09}','\u{AB0E}'),('\u{AB11}','\u{AB16}'),('\u{AB20}','\u{AB26}'),('\u{AB28}','\u{AB2E}'),('\u{AB30}','\u{AB5A}'),('\u{AB5C}','\u{AB5F}'),'\u{AB64}','\u{AB65}',('\u{ABC0}','\u{ABEA}'),'\u{ABEC}','\u{ABED}',('\u{ABF0}','\u{ABF9}'),('\u{AC00}','\u{D7A3}'),('\u{D7B0}','\u{D7C6}'),('\u{D7CB}','\u{D7FB}'),('\u{F900}','\u{FA6D}'),('\u{FA70}','\u{FAD9}'),('\u{FB00}','\u{FB06}'),('\u{FB13}','\u{FB17}'),('\u{FB1D}','\u{FB28}'),('\u{FB2A}','\u{FB36}'),('\u{FB38}','\u{FB3C}'),'\u{FB3E}','\u{FB40}','\u{FB41}','\u{FB43}','\u{FB44}',('\u{FB46}','\u{FBB1}'),('\u{FBD3}','\u{FD3D}'),('\u{FD50}','\u{FD8F}'),('\u{FD92}','\u{FDC7}'),('\u{FDF0}','\u{FDFB}'),('\u{FE00}','\u{FE0F}'),('\u{FE20}','\u{FE2D}'),'\u{FE33}','\u{FE34}',('\u{FE4D}','\u{FE4F}'),('\u{FE70}','\u{FE74}'),('\u{FE76}','\u{FEFC}'),('\u{FF10}','\u{FF19}'),('\u{FF21}','\u{FF3A}'),'\u{FF3F}',('\u{FF41}','\u{FF5A}'),('\u{FF66}','\u{FFBE}'),('\u{FFC2}','\u{FFC7}'),('\u{FFCA}','\u{FFCF}'),('\u{FFD2}','\u{FFD7}'),('\u{FFDA}','\u{FFDC}')])
    }
}

struct LineOrientedReader<I> {
    chars: I,
    curr_char: Option<char>,
    next_char: Option<char>,
    curr_posn: Posn
}

impl<I> LineOrientedReader<I> where I: Iterator<Item=char> {
    pub fn new(mut chars: I) -> LineOrientedReader<I> {
        let curr_char = chars.next();
        let next_char = if curr_char.is_some() { chars.next() } else { None };
        LineOrientedReader {
            chars: chars,
            curr_char: curr_char,
            next_char: next_char,
            curr_posn: Posn {
                offset: 0,
                line: 0,
                column: 0
            }
        }
    }

    pub fn curr_char(&mut self) -> Option<char> { self.curr_char }
    pub fn curr_posn(&mut self) -> Posn { self.curr_posn }
    pub fn next_char(&mut self) -> Option<char> { self.next_char }

    pub fn bump(&mut self) {
        let curr_char = self.next_char;
        let next_char = if curr_char.is_some() { self.chars.next() } else { None };

        self.curr_char = curr_char;
        self.next_char = next_char;

        if (curr_char == Some('\r') && next_char != Some('\n')) ||
           curr_char == Some('\n') ||
           curr_char == Some('\u{2028}') ||
           curr_char == Some('\u{2029}') {
            self.curr_posn.line += 1;
            self.curr_posn.column = 0;
        } else {
            self.curr_posn.column += 1;
        }

        self.curr_posn.offset += 1;
    }
}

// test case: x=0;y=g=1;alert(eval("while(x)break\n/y/g.exec('y')"))
//       see: https://groups.google.com/d/msg/mozilla.dev.tech.js-engine.internals/2JLH5jRcr7E/Mxc7ZKc5r6sJ

struct TokenBuffer {
    tokens: LinkedList<Token>
}

impl TokenBuffer {
    fn new() -> TokenBuffer {
        TokenBuffer {
            tokens: LinkedList::new()
        }
    }

    fn is_empty(&mut self) -> bool {
        self.tokens.len() == 0
    }

    fn push_token(&mut self, token: Token) {
        assert!(self.tokens.len() == 0);
        self.tokens.push_back(token);
    }

    fn read_token(&mut self) -> Token {
        assert!(self.tokens.len() > 0);
        self.tokens.pop_front().unwrap()
    }

    fn peek_token(&mut self) -> &Token {
        assert!(self.tokens.len() > 0);
        self.tokens.front().unwrap()
    }

    fn unread_token(&mut self, token: Token) {
        assert!(self.tokens.len() < 3);
        self.tokens.push_front(token);
    }
}

macro_rules! reserved_words {
    [ $( ( $key:expr, $val:ident ) ),* ] => {
        {
            let mut temp_map = HashMap::new();
            $(
                temp_map.insert($key, ReservedWord::$val);
            )*
            temp_map
        }
    };
}

pub struct Lexer<I> {
    reader: LineOrientedReader<I>,
    cx: Rc<Cell<Context>>,
    lookahead: TokenBuffer,
    reserved: HashMap<&'static str, ReservedWord>
}

impl<I> Lexer<I> where I: Iterator<Item=char> {
    // constructor

    pub fn new(chars: I, cx: Rc<Cell<Context>>) -> Lexer<I> {
        let mut reserved = reserved_words![
            ("null",     Null),     ("true",       True),       ("false",    False),
            ("break",    Break),    ("case",       Case),       ("catch",    Catch),
            ("class",    Class),    ("const",      Const),      ("continue", Continue),
            ("debugger", Debugger), ("default",    Default),    ("delete",   Delete),
            ("do",       Do),       ("else",       Else),       ("export",   Export),
            ("extends",  Extends),  ("finally",    Finally),    ("for",      For),
            ("function", Function), ("if",         If),         ("import",   Import),
            ("in",       In),       ("instanceof", Instanceof), ("new",      New),
            ("return",   Return),   ("super",      Super),      ("switch",   Switch),
            ("this",     This),     ("throw",      Throw),      ("try",      Try),
            ("typeof",   Typeof),   ("var",        Var),        ("void",     Void),
            ("while",    While),    ("with",       With),       ("yield",    Yield),
            ("enum",     Enum) //,  ("await",      Await)
        ];
        Lexer {
            reader: LineOrientedReader::new(chars),
            cx: cx,
            lookahead: TokenBuffer::new(),
            reserved: reserved
        }
    }

    // public methods

    pub fn peek_token(&mut self) -> Result<&Token, LexError> {
        if self.lookahead.is_empty() {
            let token = try!(self.read_next_token());
            self.lookahead.push_token(token);
        }
        Ok(self.lookahead.peek_token())
    }

    pub fn read_token(&mut self) -> Result<Token, LexError> {
        if self.lookahead.is_empty() {
            self.read_next_token()
        } else {
            Ok(self.lookahead.read_token())
        }
    }

    pub fn unread_token(&mut self, token: Token) {
        self.lookahead.unread_token(token);
    }

    // private methods

    fn bump_until<F>(&mut self, pred: &F)
      where F: Fn(char) -> bool
    {
        loop {
            match self.reader.curr_char() {
                Some(ch) if pred(ch) => return,
                None => return,
                _ => ()
            }
            self.bump();
        }
    }

    fn take_until<F>(&mut self, s: &mut String, pred: &F)
      where F: Fn(char) -> bool
    {
        loop {
            match self.reader.curr_char() {
                Some(ch) if pred(ch) => return,
                Some(ch) => { s.push(ch); }
                None => return,
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.bump();
        self.bump();
        self.bump_until(&|ch| ch.is_es_newline());
    }

    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        self.bump();
        self.bump();
        self.bump_until(&|ch| ch == '*');
        if self.reader.curr_char() == None {
            return Err(LexError::UnexpectedEOF);
        }
        self.bump();
        self.bump();
        Ok(())
    }

    fn div_or_regexp(&mut self) -> Result<Token, LexError> {
        if self.cx.get().is_operator() {
            self.bump();
            Ok(Token::Slash)
        } else {
            self.regexp()
        }
    }

    fn regexp(&mut self) -> Result<Token, LexError> {
        self.bump();
        let mut s = String::new();
        while self.reader.curr_char() != Some('/') {
            try!(self.regexp_char(&mut s));
        }
        self.bump();
        Ok(Token::RegExp(s))
    }

    fn regexp_char(&mut self, s: &mut String) -> Result<(), LexError> {
        match self.reader.curr_char() {
            Some('\\') => self.regexp_backslash(s),
            Some('[') => self.regexp_class(s),
            Some(ch) if ch.is_es_newline() => Err(LexError::UnexpectedChar(ch)),
            Some(ch) => { self.bump(); s.push(ch); Ok(()) },
            None => Err(LexError::UnexpectedEOF)
        }
    }

    fn regexp_backslash(&mut self, s: &mut String) -> Result<(), LexError> {
        s.push('\\');
        self.bump();
        match self.reader.curr_char() {
            Some(ch) if ch.is_es_newline() => Err(LexError::UnexpectedChar(ch)),
            Some(ch) => { self.bump(); s.push(ch); Ok(()) },
            None => Err(LexError::UnexpectedEOF)
        }
    }

    fn regexp_class(&mut self, s: &mut String) -> Result<(), LexError> {
        self.bump();
        s.push('[');
        while self.reader.curr_char().map_or(false, |ch| ch != ']') {
            try!(self.regexp_class_char(s));
        }
        self.bump();
        s.push(']');
        Ok(())
    }

    fn regexp_class_char(&mut self, s: &mut String) -> Result<(), LexError> {
        match self.reader.curr_char() {
            Some('\\') => self.regexp_backslash(s),
            Some(ch) => { self.bump(); s.push(ch); Ok(()) },
            None => Err(LexError::UnexpectedEOF)
        }
    }

    fn if_assign(&mut self, cons: Token, alt: Token) -> Token {
        self.reader.bump();
        if self.reader.curr_char() == Some('=') { self.reader.bump(); cons } else { alt }
    }

    fn if_equality(&mut self, zero: Token, one: Token, two: Token) -> Token {
        self.bump();
        if self.reader.curr_char() == Some('=') {
            self.bump();
            if self.reader.curr_char() == Some('=') {
                self.bump();
                two
            } else {
                one
            }
        } else {
            zero
        }
    }

    fn lt(&mut self) -> Token {
        self.bump();
        match self.reader.curr_char() {
            Some('<') => {
                self.bump();
                if self.reader.curr_char() == Some('=') {
                    self.bump();
                    Token::LShiftAssign
                } else {
                    Token::LShift
                }
            },
            Some('=') => { self.bump(); Token::LEq }
            _ => Token::LAngle
        }
    }

    fn gt(&mut self) -> Token {
        self.bump();
        match self.reader.curr_char() {
            Some('>') => {
                self.bump();
                match self.reader.curr_char() {
                    Some('>') => {
                        self.bump();
                        if self.reader.curr_char() == Some('=') {
                            self.bump();
                            Token::URShiftAssign
                        } else {
                            Token::URShift
                        }
                    },
                    Some('=') => { self.bump(); Token::RShiftAssign },
                    _ => Token::RShift
                }
            },
            Some('=') => { self.bump(); Token::GEq },
            _ => Token::RAngle
        }
    }

    fn plus(&mut self) -> Token {
        self.bump();
        match self.reader.curr_char() {
            Some('+') => { self.bump(); Token::Inc },
            Some('=') => { self.bump(); Token::PlusAssign },
            _ => Token::Plus
        }
    }

    fn minus(&mut self) -> Token {
        self.bump();
        match self.reader.curr_char() {
            Some('-') => { self.bump(); Token::Dec },
            Some('=') => { self.bump(); Token::MinusAssign },
            _ => Token::Minus
        }
    }

    fn decimal_digits_into(&mut self, s: &mut String) -> Result<(), LexError> {
        match self.reader.curr_char() {
            Some(ch) if !ch.is_digit(10) => return Err(LexError::UnexpectedChar(ch)),
            None => return Err(LexError::UnexpectedEOF),
            _ => ()
        }
        let mut s = String::new();
        self.take_until(&mut s, &|ch| !ch.is_digit(10));
        Ok(())
    }

    fn decimal_digits(&mut self) -> Result<String, LexError> {
        let mut s = String::new();
        try!(self.decimal_digits_into(&mut s));
        Ok(s)
    }

    fn exp_part(&mut self) -> Result<Option<String>, LexError> {
        match self.reader.curr_char() {
            Some(ch@'e') | Some(ch@'E') => {
                let mut s = String::new();
                s.push(ch);
                self.bump();
                match self.reader.curr_char() {
                    Some('+') | Some('-') => { s.push(self.eat().unwrap()); }
                    _ => ()
                }
                try!(self.decimal_digits_into(&mut s));
                Ok(Some(s))
            }
            _ => Ok(None)
        }
    }

    fn decimal_int(&mut self) -> Result<String, LexError> {
        let mut s = String::new();
        match self.reader.curr_char() {
            Some('0') => { s.push('0'); return Ok(s); }
            Some(ch) if ch.is_digit(10) => { self.bump(); s.push(ch); }
            Some(ch) => return Err(LexError::UnexpectedChar(ch)),
            None => return Err(LexError::UnexpectedEOF)
        }
        self.take_until(&mut s, &|ch| !ch.is_digit(10));
        Ok(s)
    }

    fn int<F, G>(&mut self, pred: &F, cons: &G) -> Result<Token, LexError>
      where F: Fn(char) -> bool,
            G: Fn(char, String) -> Token
    {
        assert!(self.reader.curr_char().is_some());
        assert!(self.reader.next_char().is_some());
        let mut s = String::new();
        self.bump();
        let flag = self.eat().unwrap();
        try!(self.digit_into(&mut s, pred));
        while self.reader.curr_char().map_or(false, |ch| pred(ch)) {
            s.push(self.eat().unwrap());
        }
        Ok(cons(flag, s))
    }

    fn hex_int(&mut self) -> Result<Token, LexError> {
        self.int(&|ch| ch.is_es_hex_digit(), &Token::HexInt)
    }

    fn oct_int(&mut self) -> Result<Token, LexError> {
        self.int(&|ch| ch.is_es_oct_digit(), &|ch, s| Token::OctalInt(Some(ch), s))
    }

    fn deprecated_oct_int(&mut self) -> Token {
        let mut s = String::new();
        while self.reader.curr_char().map_or(false, |ch| ch.is_es_oct_digit()) {
            s.push(self.eat().unwrap());
        }
        Token::OctalInt(None, s)
    }

    fn number(&mut self) -> Result<Token, LexError> {
        if self.reader.curr_char() == Some('.') {
            let frac = try!(self.decimal_digits());
            let exp = try!(self.exp_part());
            return Ok(Token::Float(None, Some(frac), exp));
        }
        if self.reader.curr_char() == Some('0') {
            match self.reader.next_char() {
                Some('x') | Some('X') => return self.hex_int(),
                Some('o') | Some('O') => return self.oct_int(),
                Some(ch) if ch.is_digit(10) => return Ok(self.deprecated_oct_int()),
                _ => {
                    self.bump();
                    return Ok(Token::DecimalInt(String::from_str("0")));
                }
            }
        }
        let pos = try!(self.decimal_int());
        let dot;
        let frac = if self.reader.curr_char() == Some('.') {
            dot = true;
            self.bump();
            match self.reader.curr_char() {
                Some(ch) if ch.is_digit(10) => Some(try!(self.decimal_digits())),
                _ => None
            }
        } else {
            dot = false;
            None
        };
        let exp = try!(self.exp_part());
        if dot { Ok(Token::Float(Some(pos), frac, exp)) } else { Ok(Token::DecimalInt(pos)) }
    }

    fn string(&mut self) -> Result<Token, LexError> {
        let mut s = String::new();
        loop {
            assert!(self.reader.curr_char().is_some());
            let quote = self.eat().unwrap();
            self.take_until(&mut s, &|ch| {
                ch == quote ||
                ch == '\\' ||
                ch.is_es_newline()
            });
            match self.reader.curr_char() {
                Some('\\') => { try!(self.string_escape(&mut s)); },
                Some(ch) => {
                    if ch.is_es_newline() {
                        return Err(LexError::UnexpectedChar(ch));
                    }
                    self.bump();
                },
                None => return Err(LexError::UnexpectedEOF)
            }
        }
        Ok(Token::String(s))
    }

    fn string_escape(&mut self, s: &mut String) -> Result<(), LexError> {
        s.push(self.eat().unwrap());
        match self.reader.curr_char() {
            Some('0') => {
                self.bump();
                let mut i = 0_u32;
                while self.reader.curr_char().map_or(false, |ch| ch.is_digit(10)) && i < 3 {
                    s.push(self.eat().unwrap());
                }
            },
            Some(ch) if ch.is_es_single_escape_char() => {
                s.push(self.eat().unwrap());
            },
            Some('x') => {
                self.bump();
                s.push('x');
                try!(self.hex_digit_into(s));
                try!(self.hex_digit_into(s));
            },
            Some('u') => {
                self.bump();
                if self.reader.curr_char() == Some('{') {
                    s.push('{');
                    self.bump();
                    try!(self.hex_digit_into(s));
                    while self.reader.curr_char() != Some('}') {
                        try!(self.hex_digit_into(s));
                    }
                    s.push('}');
                    self.bump();
                } else {
                    for i in 0..4 {
                        try!(self.hex_digit_into(s));
                    }
                }
            },
            Some(ch) if ch.is_es_newline() => {
                self.newline_into(s);
            },
            Some(ch) => {
                self.bump();
                s.push(ch);
            },
            None => () // error will be reported from caller
        }
        Ok(())
    }

    fn digit_into<F>(&mut self, s: &mut String, pred: &F) -> Result<(), LexError>
      where F: Fn(char) -> bool
    {
        match self.reader.curr_char() {
            Some(ch) if pred(ch) => {
                self.bump();
                s.push(ch);
                Ok(())
            },
            Some(ch) => Err(LexError::InvalidDigit(ch)),
            None => Err(LexError::UnexpectedEOF)
        }
    }

    fn oct_digit_into(&mut self, s: &mut String) -> Result<(), LexError> {
        self.digit_into(s, &|ch| ch.is_es_oct_digit())
    }

    fn hex_digit_into(&mut self, s: &mut String) -> Result<(), LexError> {
        self.digit_into(s, &|ch| ch.is_es_hex_digit())
    }

    fn word(&mut self) -> Token {
        let mut s = String::new();
        assert!(self.reader.curr_char().is_some());
        s.push(self.eat().unwrap());
        self.take_until(&mut s, &|ch| !ch.is_es_identifier_continue());
        match self.reserved.get(&s[..]) {
            Some(word) => Token::Reserved(*word),
            None => Token::Identifier(s)
        }
    }

    fn read_next_token(&mut self) -> Result<Token, LexError> {
        self.skip_whitespace();
        println!("inspecting {:?}", self.reader.curr_char());
        loop {
            match self.reader.curr_char() {
                Some('/') => {
                    match self.reader.next_char() {
                        Some('/') => self.skip_line_comment(),
                        Some('*') => { try!(self.skip_block_comment()); },
                        _ => return self.div_or_regexp()
                    }
                },
                Some('.') => {
                    match self.reader.next_char() {
                        Some(ch) if ch.is_digit(10) => { return self.number() },
                        _ => { self.reader.bump(); return Ok(Token::Dot) }
                    }
                }
                Some('{') => { self.bump(); return Ok(Token::LBrace) },
                Some('}') => { self.bump(); return Ok(Token::RBrace) },
                Some('[') => { self.bump(); return Ok(Token::LBrack) },
                Some(']') => { self.bump(); return Ok(Token::RBrack) },
                Some('(') => { self.bump(); return Ok(Token::LParen) },
                Some(')') => { self.bump(); return Ok(Token::RParen) },
                Some(';') => { self.bump(); return Ok(Token::Semi) },
                Some(':') => { self.bump(); return Ok(Token::Colon) },
                Some(',') => { self.bump(); return Ok(Token::Comma) },
                Some('<') => return Ok(self.lt()),
                Some('>') => return Ok(self.gt()),
                Some('=') => return Ok(self.if_equality(Token::Assign, Token::Eq, Token::StrictEq)),
                Some('+') => return Ok(self.plus()),
                Some('-') => return Ok(self.minus()),
                Some('*') => return Ok(self.if_assign(Token::StarAssign, Token::Star)),
                Some('%') => return Ok(self.if_assign(Token::ModAssign, Token::Mod)),
                Some('^') => return Ok(self.if_assign(Token::BitXorAssign, Token::BitXor)),
                Some('&') => {
                    self.bump();
                    match self.reader.curr_char() {
                        Some('&') => { self.bump(); return Ok(Token::LogicalAnd) },
                        _ => return Ok(Token::BitAnd)
                    }
                },
                Some('|') => {
                    self.bump();
                    match self.reader.curr_char() {
                        Some('|') => { self.bump(); return Ok(Token::LogicalOr) },
                        _ => return Ok(Token::BitOr)
                    }
                },
                Some('~') => { self.bump(); return Ok(Token::Tilde) },
                Some('!') => return Ok(self.if_equality(Token::Bang, Token::NEq, Token::StrictNEq)),
                Some('?') => { self.bump(); return Ok(Token::Question) },
                Some('"') => return self.string(),
                Some('\'') => return self.string(),
                Some(ch) if ch.is_es_newline() => {
                    self.newline();
                    if self.cx.get().is_asi_possible() {
                        return Ok(Token::Newline);
                    }
                }
                Some(ch) if ch.is_digit(10) => return self.number(),
                Some(ch) if ch.is_es_identifier_start() => return Ok(self.word()),
                Some(ch) => return Err(LexError::UnexpectedChar(ch)),
                None => return Ok(Token::EOF)
            }
        }
    }

    fn newline(&mut self) {
        assert!(self.reader.curr_char().map_or(false, |ch| ch.is_es_newline()));
        if self.reader.curr_char() == Some('\r') && self.reader.next_char() == Some('\n') {
            self.bump();
        }
        self.bump();
    }

    fn newline_into(&mut self, s: &mut String) {
        assert!(self.reader.curr_char().map_or(false, |ch| ch.is_es_newline()));
        if self.reader.curr_char() == Some('\r') && self.reader.next_char() == Some('\n') {
            s.push('\r');
            s.push('\n');
            self.bump();
            self.bump();
            return;
        }
        s.push(self.eat().unwrap());
    }

    fn is_whitespace(&mut self) -> bool {
        match self.reader.curr_char() {
            Some(ch) => ch.is_es_whitespace(),
            None => false
        }
    }

    fn bump(&mut self) {
        self.reader.bump();
    }

    fn eat(&mut self) -> Option<char> {
        let ch = self.reader.curr_char();
        self.bump();
        ch
    }

    fn skip_whitespace(&mut self) {
        while self.is_whitespace() {
            self.reader.bump();
        }
    }
}

impl<I> Iterator for Lexer<I> where I: Iterator<Item=char> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        match self.read_token() {
            Ok(Token::EOF) => None,
            Ok(t) => Some(t),
            Err(_) => None
        }
    }
}
