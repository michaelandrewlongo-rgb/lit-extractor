use crate::brief::compose::BriefJson;
use crate::errors::{LitError, Result};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn render_pdf(brief: &BriefJson, out_path: &Path) -> Result<()> {
    let (doc, page1, layer1) = PdfDocument::new(&brief.slug, Mm(210.0), Mm(297.0), "Layer 1");
    let layer = doc.get_page(page1).get_layer(layer1);
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| LitError::Pipeline(format!("pdf font error: {e}")))?;

    let mut y = 280.0;
    layer.use_text(format!("Neurosurgery Brief: {}", brief.query), 16.0, Mm(10.0), Mm(y), &font);
    y -= 10.0;
    layer.use_text(
        format!("Generated: {}", brief.generated_at.to_rfc3339()),
        10.0,
        Mm(10.0),
        Mm(y),
        &font,
    );
    y -= 12.0;

    layer.use_text("Key Takeaways", 13.0, Mm(10.0), Mm(y), &font);
    y -= 8.0;
    for (idx, t) in brief.takeaways.iter().enumerate().take(8) {
        let line = format!("{}. {}", idx + 1, t.text);
        layer.use_text(line.chars().take(120).collect::<String>(), 9.0, Mm(12.0), Mm(y), &font);
        y -= 6.0;
        if y < 30.0 {
            break;
        }
    }

    let file = File::create(out_path)?;
    doc.save(&mut BufWriter::new(file))
        .map_err(|e| LitError::Pipeline(format!("pdf save error: {e}")))?;
    Ok(())
}
