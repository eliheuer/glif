use std::collections::HashSet;

use super::prelude::*;
use glifparser::{glif::contour::MFEKContourCommon, MFEKPointData};
use MFEKmath::rect::FlipIfRequired;
use glifparser::glif::MFEKOutline;
use glifrenderer::{
    constants::{OUTLINE_STROKE, OUTLINE_STROKE_THICKNESS},
    points::draw_point,
};
use skia_safe::dash_path_effect;

#[derive(Clone, Debug, Default)]
pub struct SelectionBox {
    mouse_info: MouseInfo,
    corner: Option<(f32, f32)>,
    selected: HashSet<(usize, usize)>,
}

impl SelectionBox {
    pub fn new(mouse_info: MouseInfo) -> Self {
        SelectionBox {
            mouse_info,
            corner: None,
            selected: HashSet::new(),
        }
    }

    pub fn mouse_moved(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        self.corner = Some(mouse_info.position);
        let selected = {
            // we get out starting mouse position, and our current mouse position
            let c1 = self.mouse_info.position;
            let c2 = mouse_info.position;

            let rect = Rect::from_point_and_size(
                (c1.0 as f32, c1.1 as f32),
                ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
            );

            build_box_selection(rect, &v.get_active_layer_ref().outline)
        };

        self.selected = selected
    }

    pub fn mouse_released(&mut self, v: &mut Editor, _i: &mut Interface, mouse_info: MouseInfo) {
        if mouse_info.button == self.mouse_info.button {
            if mouse_info.modifiers.shift {
                v.selected = &v.selected ^ &self.selected;
            } else if self.selected.len() == 1 {
                for (cidx, pidx) in &self.selected {
                    v.contour_idx = Some(*cidx);
                    v.point_idx = Some(*pidx);
                }
            } else {
                if self.selected.len() == 1 {
                    for (cidx, pidx) in &self.selected {
                        v.contour_idx = Some(*cidx);
                        v.point_idx = Some(*pidx);
                    }
                } else {
                    v.selected = self.selected.clone();
                }
            }
            v.pop_behavior();
        }
    }

    pub fn draw_box_impl(i: &Interface, canvas: &mut Canvas, (c1, c2): ((f32, f32), (f32, f32))) {
        let mut path = Path::new();
        let mut paint = Paint::default();
        let rect = Rect::from_point_and_size(
            (c1.0 as f32, c1.1 as f32),
            ((c2.0 - c1.0) as f32, (c2.1 - c1.1) as f32),
        );
        path.add_rect(rect, None);
        path.close();
        paint.set_color(OUTLINE_STROKE);
        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(OUTLINE_STROKE_THICKNESS * (1. / i.viewport.factor));
        let dash_offset = (1. / i.viewport.factor) * 2.;
        paint.set_path_effect(dash_path_effect::new(&[dash_offset, dash_offset], 0.0));
        canvas.draw_path(&path, &paint);
    }

    pub fn draw_box(&self, i: &Interface, canvas: &mut Canvas) {
        let c1 = self.mouse_info.position;
        let c2 = self.corner.unwrap_or(self.mouse_info.position);
        Self::draw_box_impl(i, canvas, (c1, c2));
    }

    pub fn draw_selected(&self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        for (ci, pi) in &self.selected {
            let (ci, pi) = (*ci, *pi);

            {
                let layer = v.get_active_layer_ref();
                let point = get_point!(layer, ci, pi).unwrap();
                draw_point(
                    &i.viewport,
                    point,
                    None,
                    true,
                    canvas,
                )
            }
        }
    }
}

impl ToolBehavior for SelectionBox {
    #[rustfmt::skip]
    fn event(&mut self, v: &mut Editor, i: &mut Interface, event: EditorEvent) {
        if let EditorEvent::MouseEvent { mouse_info, event_type } = event {
            match event_type {
                MouseEventType::Released => self.mouse_released(v, i, mouse_info),
                MouseEventType::Moved => self.mouse_moved(v, i, mouse_info),
                _ => (),
            }
        }
    }

    fn draw(&mut self, v: &Editor, i: &Interface, canvas: &mut Canvas) {
        self.draw_box(i, canvas);
        self.draw_selected(v, i, canvas)
    }
}

pub fn build_box_selection(
    mut rect: SkRect,
    outline: &MFEKOutline<MFEKPointData>,
) -> HashSet<(usize, usize)> {
    rect.flip_if_required();

    let mut selected = HashSet::new();
    for (cidx, contour) in outline.iter().enumerate() {
        for (pidx, point) in contour.inner().iter().enumerate() {
            if rect.contains(SkPoint::from((point.x(), point.y()))) {
                selected.insert((cidx, pidx));
            }
        }
    }

    selected
}