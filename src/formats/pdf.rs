use ::OutputDevice;
use ::util::geom::Point;

pub struct PdfWriter {
    size: (f32, f32),
    buffer: String,
    line_buffer: String,
    conversion: f32,
    start: bool,
}

impl PdfWriter {
    pub fn new(_units: &str, conversion_factor: f32) -> PdfWriter {
        PdfWriter {
            size: (800.0, 800.0),
            buffer: String::new(),
            line_buffer: "0 w\n".into(),
            conversion: conversion_factor,
            start: false
        }
    }

    pub fn write_out(mut self, path: &str) {
        use std::io::Write;
        use std::fs::File;

        self.buffer = format!(r#"%PDF-1.6
1 0 obj <</Type /Catalog /Pages 2 0 R>>
endobj
2 0 obj <</Type /Pages /Kids [3 0 R] /Count 1>>
endobj
3 0 obj<</Type /Page /Parent 2 0 R /Contents 4 0 R /MediaBox [0 0 {} {}]>>
endobj
"#, self.size.0, self.size.1);

        let length_of_line_buffer = self.line_buffer.len();
        let location_of_lines = self.buffer.len();

        self.buffer.push_str("4 0 obj\n");
        self.buffer.push_str(&format!("<</Length {}>>\n", length_of_line_buffer));

        self.buffer.push_str("stream\n");
        self.buffer.push_str(&self.line_buffer);
        if length_of_line_buffer != 0 {
            self.buffer.push_str("S\n\n");
        }

        self.buffer.push_str("endstream\nendobj\n");
        let xref_pos = self.buffer.len() + 1; // account for the newline
        self.buffer.push_str(&format!(r#"
xref
0 5
0000000000 65535 f
0000000009 00000 n
0000000056 00000 n
0000000111 00000 n
{:010} 00000 n
trailer <</Size 5 /Root 1 0 R>>
startxref
{}
%%EOF"#, location_of_lines, xref_pos));
        let mut f = File::create(path).unwrap();
        f.write(self.buffer.as_bytes()).unwrap();
    }
}

impl PdfWriter {
    fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        (x * self.conversion, self.size.1 - y * self.conversion)
    }
}

impl OutputDevice for PdfWriter {
    fn start_line(&mut self) {
        self.start = true;
    }

    fn add_point(&mut self, Point{x, y}: Point) {
        let (x, y) = self.transform_point(x, y);
        self.line_buffer.push_str(&format!("{} {} {}\n", x, y, if self.start { "m" } else { "l" }));
        self.start = false;
    }

    fn end_line(&mut self) { }

    fn set_size(&mut self, w: f32, h: f32) {
        self.size = (w * self.conversion, h * self.conversion);
    }
}
