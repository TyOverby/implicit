use ::OutputDevice;
use ::util::geom::Point;

pub struct PdfWriter {
    size: (f32, f32),
    line_buffer: String,
    conversion: f32,
    start: bool,
}

fn append_line<S: AsRef<str>>(buffer: &mut String, line: S) {
    buffer.push_str(line.as_ref());
    buffer.push_str("\r\n");
}

impl PdfWriter {
    pub fn new(_units: &str, conversion_factor: f32) -> PdfWriter {
        PdfWriter {
            size: (800.0, 800.0),
            line_buffer: String::new(),
            conversion: conversion_factor,
            start: false
        }
    }

    pub fn write_out(mut self, path: &str) {
        use std::io::Write;
        use std::fs::File;

        fn write_header(buffer: &mut String, offsets: &mut Vec<usize>, (width, height): (f32, f32)) {
            append_line(buffer, "%PDF-1.6");

            offsets.push(buffer.len());
            append_line(buffer, "1 0 obj");
            append_line(buffer, "<</Type /Catalog /Pages 2 0 R>>");
            append_line(buffer, "endobj");

            offsets.push(buffer.len());
            append_line(buffer, "2 0 obj");
            append_line(buffer, "<</Type /Pages /Kids [3 0 R] /Count 1>>");
            append_line(buffer, "endobj");

            offsets.push(buffer.len());
            append_line(buffer, "3 0 obj");
            append_line(buffer, format!("<</Type /Page /Parent 2 0 R /Contents 4 0 R /MediaBox [0 0 {} {}] /Resources<<>>>>", width, height));
            append_line(buffer, "endobj");
        }

        fn write_body(buffer: &mut String, drawing_lines: &str) {
            //append_line(buffer, "/DeviceRGB CS");
            // Set the pen width to 0
            append_line(buffer, "0 w");
            buffer.push_str(drawing_lines);
            // Final stroke
            append_line(buffer, "S");
        }

        let mut buffer = String::new();
        let mut body = String::new();
        let mut offsets = Vec::new();

        write_header(&mut buffer, &mut offsets, self.size);
        write_body(&mut body, &self.line_buffer);

        offsets.push(buffer.len());

        append_line(&mut buffer, "4 0 obj");
        append_line(&mut buffer, format!("<</length {}>>", body.len()));
        append_line(&mut buffer, "stream");
        buffer.push_str(&body);
        append_line(&mut buffer, "");
        append_line(&mut buffer, "endstream");
        append_line(&mut buffer, "endobj");

        let xref_location = buffer.len();
        // +1 because of the default empty object
        let xref_count = offsets.len() + 1; 
        append_line(&mut buffer, "xref");
        append_line(&mut buffer, format!("0 {}", xref_count));
        // The default empty object
        append_line(&mut buffer, "0000000000 65535 f");

        for offset in offsets {
            append_line(&mut buffer, format!("{:010} 00000 n", offset));
        }

        append_line(&mut buffer, format!("trailer <</Size {} /Root 1 0 R>>", xref_count));
        append_line(&mut buffer, "startxref");
        append_line(&mut buffer, format!("{}", xref_location));
        buffer.push_str("%%EOF");

        let mut f = File::create(path).unwrap();
        f.write(buffer.as_bytes()).unwrap();
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
        let cmd = if self.start { "m" } else { "l" };
        append_line(&mut self.line_buffer, format!("{} {} {}", x, y, cmd));
        self.start = false;
    }

    fn end_line(&mut self) { }

    fn set_size(&mut self, w: f32, h: f32) {
        self.size = (w * self.conversion, h * self.conversion);
    }
}
