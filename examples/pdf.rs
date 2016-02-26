extern crate lux;
extern crate implicit;

mod helper;
mod display;

use implicit::*;

fn main() {
    let mut pdf_writer = PdfWriter::new(0.0, 0.0, "cm", 1.0);
    pdf_writer.write_out("mine.pdf");
}