#[macro_escape];

use std::str::CharIterator;
use std::rt::io::file::FileWriter;
use std::rt::io::Writer;

struct ParseBranch {
    matches: ~[u8],
    result: Option<~str>,
    children: ~[ParseBranch],
}

impl ParseBranch {
    fn new() -> ParseBranch {
        ParseBranch {
            matches: ~[],
            result: None,
            children: ~[],
        }
    }
}

pub fn branchify(options: &[(&str, &str)], case_sensitive: bool) -> ~[ParseBranch] {
    let mut root = ParseBranch::new();

    fn go_down_moses(branch: &mut ParseBranch, mut chariter: CharIterator, result: &str, case_sensitive: bool) {
        match chariter.next() {
            Some(c) => {
                let first_case = if case_sensitive { c as u8 } else { c.to_ascii().to_upper().to_byte() };
                for next_branch in branch.children.mut_iter() {
                    if next_branch.matches[0] == first_case {
                        go_down_moses(next_branch, chariter, result, case_sensitive);
                        return;
                    }
                }
                let mut subbranch = ParseBranch::new();
                subbranch.matches.push(first_case);
                if !case_sensitive {
                    let second_case = c.to_ascii().to_lower().to_byte();
                    if first_case != second_case {
                        subbranch.matches.push(second_case);
                    }
                }
                branch.children.push(subbranch);
                go_down_moses(&mut branch.children[branch.children.len() - 1], chariter, result, case_sensitive);
            },
            None => {
                assert!(branch.result.is_none());
                branch.result = Some(result.to_owned());
            },
        }
    };

    for &(key, result) in options.iter() {
        go_down_moses(&mut root, key.iter(), result, case_sensitive);
    }

    root.children
}

macro_rules! branchify(
    (case sensitive, $($key:expr => $value:ident),*) => (
        ::branchify::branchify([$(($key, stringify!($value))),*], true)
    );
    (case insensitive, $($key:expr => $value:ident),*) => (
        branchify([$(($key, stringify!($value))),*], false)
    );
)

/// Prints the contents to stdout.
///
/// :param branches: the branches to search through
/// :param indent: the level of indentation (each level representing four leading spaces)
/// :param read_call: the function call to read a byte
/// :param end: the byte which marks the end of the sequence
/// :param max_len: the maximum length a value may be before giving up and returning ``None``
/// :param valid: the function call to if a byte ``b`` is valid
/// :param unknown: the expression to call for an unknown value; in this string, ``{}`` will be
///         replaced with an expression (literal or non-literal) evaluating to a ``~str`` (it is
///         ``{}`` only, not arbitrary format strings)
pub fn generate_branchified_method(
        writer: &mut FileWriter,
        branches: &[ParseBranch],
        indent: uint,
        read_call: &str,
        end: &str,
        max_len: &str,
        valid: &str,
        unknown: &str) {

    fn r(writer: &mut FileWriter, branch: &ParseBranch, prefix: &str, indent: uint, read_call: &str,
            end: &str, max_len: &str, valid: &str, unknown: &str) {
        let indentstr = " ".repeat(indent * 4);
        let w = |s: &str| {
            writer.write(indentstr.as_bytes());
            writer.write(s.as_bytes());
            writer.write(bytes!("\n"));
        };
        for &c in branch.matches.iter() {
            let next_prefix = format!("{}{}", prefix, c as char);
            w(format!("Some(b) if b == '{}' as u8 => match {} \\{", c as char, read_call));
            for b in branch.children.iter() {
                r(writer, b, next_prefix, indent + 1, read_call, end, max_len, valid, unknown);
            }
            match branch.result {
                Some(ref result) => w(format!("    Some(b) if b == SP => return Some({}),", *result)),
                None => w(format!("    Some(b) if b == SP => return Some({}),",
                                  unknown.replace("{}", format!("~\"{}\"", next_prefix)))),
            }
            w(format!("    Some(b) if {} => (\"{}\", b),", valid, next_prefix));
            w("    _ => return None,");
            w("},");
        }
    }
    let indentstr = " ".repeat(indent * 4);
    let w = |s: &str| {
        writer.write(indentstr.as_bytes());
        writer.write(s.as_bytes());
        writer.write(bytes!("\n"));
    };

    w(format!("let (s, next_byte) = match {} \\{", read_call));
    for b in branches.iter() {
        r(writer, b, "", indent + 1, read_call, end, max_len, valid, unknown);
    }
    w(format!("    Some(b) if {} => (\"\", b),", valid));
    w(       ("    _ => return None,"));
    w(       ("};"));
    w(       ("// OK, that didn't pan out. Let's read the rest and see what we get."));
    w(       ("let mut s = s.to_owned();"));
    w(       ("s.push_char(next_byte as char);"));
    w(       ("loop {"));
    w(format!("    match {} \\{", read_call));
    w(format!("        Some(b) if b == {} => return Some({}),", end, unknown.replace("{}", "s")));
    w(format!("        Some(b) if {} => \\{", valid));
    w(format!("            if s.len() == {} \\{", max_len));
    w(       ("                // Too long; bad request"));
    w(       ("                return None;"));
    w(       ("            }"));
    w(       ("            s.push_char(b as char);"));
    w(       ("        },"));
    w(       ("        _ => return None,"));
    w(       ("    }"));
    w(       ("}"));
}
