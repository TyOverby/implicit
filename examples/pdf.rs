extern crate lux;
extern crate implicit;

mod helper;
mod display;

use implicit::formats::pdf::PdfWriter;

fn main() {
    let pdf_writer = PdfWriter::new("cm", 1.0);
    pdf_writer.write_out("mine.pdf");
}
