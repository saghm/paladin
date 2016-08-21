// Adapted from [https://github.com/PistonDevelopers/conrod/blob/4683a5f00ba08454dda25127783d1334c06df516/examples/text_edit.rs]

#[macro_use] extern crate conrod;
extern crate find_folder;
extern crate piston_window;

use piston_window::{AdvancedWindow, EventLoop, OpenGL, PistonWindow, UpdateEvent};

fn main() {
    const WIDTH: u32 = 1600;
    const HEIGHT: u32 = 900;

    // Construct the window.
    let mut window: PistonWindow =
        piston_window::WindowSettings::new("Pal IDE", [WIDTH, HEIGHT])
            .opengl(OpenGL::V3_2).exit_on_esc(true).build().unwrap();
    window.set_ups(60);
    window.set_position([100, 100]);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5).for_folder("inconsolata").expect("Unable to find font folder");
    let font_path = assets.join("Inconsolata-Regular.ttf");
    ui.fonts.insert_from_file(font_path).expect("Unable to insert font file");

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    // Some starting text to edit.
    let mut text = String::new();

    while let Some(event) = window.next() {
        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| set_ui(ui.set_widgets(), &mut text));

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T { img };
                conrod::backend::piston_window::draw(c, g, primitives,
                    &mut text_texture_cache,
                    &image_map,
                    texture_from_image);
                }
        });
    }
}

fn set_ui(ref mut ui: conrod::UiCell, text: &mut String) {
    use conrod::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget_ids!{CANVAS, TEXT_EDIT};

    widget::Canvas::new().color(color::BLACK).set(CANVAS, ui);

    for edit in widget::TextEdit::new(text)
        .color(color::WHITE)
        .padded_wh_of(CANVAS, 20.0)
        .middle_of(CANVAS)
        .align_text_left()
        .line_spacing(10.0)
        .font_size(25)
        .set(TEXT_EDIT, ui)
    {
        *text = edit;
    }
}
