use super::*;
use ::Implicit;
use ::std::io::Write;
use ::std::io::Result as IoResult;

pub fn svg_header<W: Write>(w: &mut W, width: f32, height: f32, units: &str) -> IoResult<()> {
    try!(writeln!(w, r#"<?xml version="1.0" standalone="no"?>"#));
    try!(writeln!(w, r#"<svg width="{0}{2}" height="{1}{2}" version="1.1" xmlns="http://www.w3.org/2000/svg">"#, width, height, units));
    Ok(())
}

pub fn svg_footer<W: Write>(w: &mut W) -> IoResult<()> {
    writeln!(w, "</svg>")
}

/*
pub fn export_svg<'a, S: 'a, I>(objects: I, resolution: f32, simplify: bool, width: f32, height: f32, units: &str) -> String
where S: Implicit, I: Iterator<Item=&'a S> {
    let rendered = objects.map(|obj| render(obj, resolution, simplify));
    for lines in rendered {
        for v in lines {
            print!(r#"<path stroke-width="0.01px" fill="none" stroke="black" d=""#);
            let mut vi = v.into_iter();
            let first = vi.next();
            if let Some(first) = first {
                print!("M{} {} ", first.x, first.y);
            }
            for p in vi {
                print!("L {} {} ", p.x, p.y);
            }
            if let Some(first) = first {
                print!("L{} {} ", first.x, first.y);
            }
            println!("\"/>");
        }
    }
    println!(r#"</svg>"#);
    return "".into();
}
*/
