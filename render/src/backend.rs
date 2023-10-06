use pathfinder_geometry::{
    transform2d::Transform2F,
    rect::RectF,
};
use pathfinder_content::{
    fill::FillRule,
    stroke::{StrokeStyle},
    outline::Outline,
};
use pdf::{object::{Ref, XObject, ImageXObject, Resolve, Resources, MaybeRef}, content::Op};
use pdf::error::PdfError;
use font::Glyph;
use super::{FontEntry, TextSpan, Fill};
use pdf::font::Font as PdfFont;
use std::sync::Arc;

#[derive(Debug, Copy, Clone)]
pub enum BlendMode {
    Overlay,
    Darken
}

pub trait Backend {
    fn set_clip_path(&mut self, path: Option<&Outline>);
    fn draw(&mut self, outline: &Outline, mode: &DrawMode, fill_rule: FillRule, transform: Transform2F);
    fn set_view_box(&mut self, r: RectF);
    fn draw_image(&mut self, xref: Ref<XObject>, im: &ImageXObject, resources: &Resources, transform: Transform2F, mode: BlendMode, resolve: &impl Resolve);
    fn draw_inline_image(&mut self, im: &Arc<ImageXObject>, resources: &Resources, transform: Transform2F, mode: BlendMode, resolve: &impl Resolve);
    fn draw_glyph(&mut self, glyph: &Glyph, mode: &DrawMode, transform: Transform2F) {
        self.draw(&glyph.path, mode, FillRule::Winding, transform);
    }
    fn get_font(&mut self, font_ref: &MaybeRef<PdfFont>, resolve: &impl Resolve) -> Result<Option<Arc<FontEntry>>, PdfError>;
    fn add_text(&mut self, span: TextSpan);

    /// The following functions are for debugging PDF files and not relevant for rendering them.
    fn bug_text_no_font(&mut self, data: &[u8]) {}
    fn bug_text_invisible(&mut self, text: &str) {}
    fn bug_postscript(&mut self, data: &[u8]) {}
    fn bug_op(&mut self, op_nr: usize) {}
    fn inspect_op(&mut self, op: &Op) {}
}
#[derive(Clone)]
pub enum DrawMode {
    Fill(Fill, f32),
    Stroke(Fill, f32, Stroke),
    FillStroke(Fill, f32, Fill, f32, Stroke),
}
#[derive(Clone, Debug)]
pub struct Stroke {
    pub dash_pattern: Option<(Vec<f32>, f32)>,
    pub style: StrokeStyle,
}
