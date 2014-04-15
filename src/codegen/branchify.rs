#![macro_escape]

use std::str::Chars;
use std::io::IoResult;

pub struct ParseBranch {
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

    fn go_down_moses(branch: &mut ParseBranch, mut chariter: Chars, result: &str, case_sensitive: bool) {
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
        go_down_moses(&mut root, key.chars(), result, case_sensitive);
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
        writer: &mut Writer,
        branches: &[ParseBranch],
        indent: uint,
        read_call: &str,
        end: &str,
        max_len: &str,
        valid: &str,
        unknown: &str) -> IoResult<()> {

    fn r(writer: &mut Writer, branch: &ParseBranch, prefix: &str, indent: uint, read_call: &str,
            end: &str, max_len: &str, valid: &str, unknown: &str) -> IoResult<()> {
        let indentstr = " ".repeat(indent * 4);
        macro_rules! w (
            ($s:expr) => {
                try!(write!(writer, "{}{}\n", indentstr, $s))
            }
        )
        for &c in branch.matches.iter() {
            let next_prefix = format!("{}{}", prefix, c as char);
            w!(format!("Ok(b) if b == '{}' as u8 => match {} \\{", c as char, read_call));
            for b in branch.children.iter() {
                try!(r(writer, b, next_prefix, indent + 1, read_call, end, max_len, valid, unknown));
            }
            match branch.result {
                Some(ref result) =>
                    w!(format!("    Ok(b) if b == SP => return Ok({}),", *result)),
                None => w!(format!("    Ok(b) if b == SP => return Ok({}),",
                                  unknown.replace("{}", format!("~\"{}\"", next_prefix)))),
            }
            w!(format!("    Ok(b) if {} => (\"{}\", b),", valid, next_prefix));
            w!("    Ok(_) => return Err(::std::io::IoError { kind: ::std::io::OtherIoError, desc: \"bad value\", detail: None }),");
            w!("    Err(err) => return Err(err),");
            w!("},");
        }
        Ok(())
    }
    let indentstr = " ".repeat(indent * 4);
    macro_rules! w (
        ($s:expr) => {
            try!(write!(writer, "{}{}\n", indentstr, $s))
        }
    )

    w!(format!("let (s, next_byte) = match {} \\{", read_call));
    for b in branches.iter() {
        try!(r(writer, b, "", indent + 1, read_call, end, max_len, valid, unknown));
    }
    w!(format!("    Ok(b) if {} => (\"\", b),", valid));
    w!(       ("    Ok(_) => return Err(::std::io::IoError { kind: ::std::io::OtherIoError, desc: \"bad value\", detail: None }),"));
    w!(       ("    Err(err) => return Err(err),"));
    w!(       ("};"));
    w!(       ("// OK, that didn't pan out. Let's read the rest and see what we get."));
    w!(       ("let mut sb = StrBuf::from_owned_str(s.to_owned());"));
    w!(       ("sb.push_char(next_byte as char);"));
    w!(       ("loop {"));
    w!(format!("    match {} \\{", read_call));
    w!(format!("        Ok(b) if b == {} => return Ok({}),", end, unknown.replace("{}", "sb.into_owned()")));
    w!(format!("        Ok(b) if {} => \\{", valid));
    w!(format!("            if sb.len() == {} \\{", max_len));
    w!(       ("                // Too long; bad request"));
    w!(       ("                return Err(::std::io::IoError { kind: ::std::io::OtherIoError, desc: \"too long, bad request\", detail: None });"));
    w!(       ("            }"));
    w!(       ("            sb.push_char(b as char);"));
    w!(       ("        },"));
    w!(       ("        Ok(_) => return Err(::std::io::IoError { kind: ::std::io::OtherIoError, desc: \"bad value\", detail: None }),"));
    w!(       ("        Err(err) => return Err(err),"));
    w!(       ("    }"));
    w!(       ("}"));
    Ok(())
}
