use ::OutputDevice;
use ::util::geom::Point;

pub struct SvgWriter {
    buffer: String,
    conversion: f32,
}

impl SvgWriter {
    pub fn new(width: f32, height: f32, units: &str, conversion_factor: f32) -> SvgWriter {
        let mut s = String::from(r#"<?xml version="1.0" standalone="no"?>"#);
        s.push_str(&format!(r#"<svg width="{0}{2}" height="{1}{2}" viewbox="0 0 {0} {1}" version="1.1" xmlns="http://www.w3.org/2000/svg">"#, width, height, units));
        SvgWriter {
            buffer: s,
            conversion: conversion_factor,
        }
    }

    pub fn write_out(mut self, path: &str) {
        use std::io::Write;
        use std::fs::File;
        self.buffer.push_str("</svg>");
        let mut f = File::create(path).unwrap();
        f.write(self.buffer.as_bytes()).unwrap();
    }
}

impl OutputDevice for SvgWriter {
    fn start_line(&mut self) {
        self.buffer.push_str("\n");
        self.buffer.push_str(r#"<path fill="none" stroke-width="0.01px" stroke="black" d="M"#);
    }

    fn add_point(&mut self, Point{x, y}: Point) {
        self.buffer.push_str(&format!("{} {} L", x * self.conversion, y * self.conversion));
    }

    fn end_line(&mut self) {
        self.buffer.pop();
        self.buffer.push_str(r#""/>"#);
    }
}
