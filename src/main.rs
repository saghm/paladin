// Adapted from [https://github.com/PistonDevelopers/conrod/blob/4683a5f00ba08454dda25127783d1334c06df516/examples/text_edit.rs]

#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate pal;
extern crate piston_window;

use std::thread;
use std::sync::{Arc, RwLock};

use pal::Event;
use piston_window::{AdvancedWindow, EventLoop, OpenGL, PistonWindow, UpdateEvent};

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 1400;

fn main() {
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

    let mut editor_text = String::new();
    let console_text = Arc::new(RwLock::new(String::new()));
    let mut input_text = String::new();

    while let Some(event) = window.next() {
        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| set_ui(&mut ui.set_widgets(), &mut editor_text, console_text.clone(), &mut input_text));

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

fn set_ui(ui: &mut conrod::UiCell, editor_text: &mut String, console_text: Arc<RwLock<String>>, input_text: &mut String) {
    use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    widget_ids! [
        CANVAS,
        RUN_BUTTON,
        EDITOR_COLOR,
        EDITOR,
        EDITOR_SCROLL,
        SEPARATOR,
        CONSOLE_COLOR,
        CONSOLE,
        INPUT_COLOR,
        INPUT
    ];

    widget::Canvas::new().color(color::DARK_CHARCOAL).scroll_kids().set(CANVAS, ui);

    let dim = ui.window_dim();
    let canvas_width = dim[0] - 150.0;
    let canvas_height = dim[1];

    widget::Rectangle::fill_with([canvas_width, canvas_height * 9.0 / 14.0], color::BLACK)
        .top_left_with_margin_on(CANVAS, 10.0)
        .set(EDITOR_COLOR, ui);

    for edit in widget::TextEdit::new(editor_text)
        .color(color::WHITE)
        .padded_w_of(EDITOR_COLOR, 5.0)
        .padded_h_of(EDITOR_COLOR, 2.0)
        .x_y_relative_to(EDITOR_COLOR, 0.0, 0.0)
        .align_text_left()
        .line_spacing(10.0)
        .font_size(25)
        .set(EDITOR, ui)
    {
        *editor_text = edit;
    }

    if widget::Button::new()
        .top_right_with_margin_on(CANVAS, 10.0)
        .w_h(120.0, canvas_height / 35.0)
        .label("Run code")
        .label_font_size(23)
        .set(RUN_BUTTON, ui)
        .into_iter().was_clicked()
    {
        console_text.write().unwrap().clear();
        run_program(editor_text.clone(), console_text.clone());
    }


    widget::Rectangle::fill_with([canvas_width, canvas_height * 3.0 / 56.0], color::DARK_BLUE)
        .down_from(EDITOR_COLOR, 10.0)
        .set(SEPARATOR, ui);

    widget::Rectangle::fill_with([canvas_width, canvas_height / 4.0], color::BLACK)
        .down_from(SEPARATOR, 10.0)
        .set(CONSOLE_COLOR, ui);

    widget::Text::new(&*console_text.read().unwrap())
        .color(color::WHITE)
        .padded_w_of(CONSOLE_COLOR, 5.0)
        .padded_h_of(CONSOLE_COLOR, 2.0)
        .x_y_relative_to(CONSOLE_COLOR, 0.0, 0.0)
        .align_text_left()
        .line_spacing(10.0)
        .font_size(25)
        .set(CONSOLE, ui);

    for edit in widget::TextBox::new(input_text)
        .w_h(canvas_width, canvas_height / 35.0)
        .down_from(CONSOLE_COLOR, 10.0)
        .font_size(25)
        .set(INPUT, ui)
        {
            match edit {
                widget::text_box::Event::Update(new_str) => *input_text = new_str,
                widget::text_box::Event::Enter => input_text.clear(),
            }
        }
}

fn run_program(program: String, console_text: Arc<RwLock<String>>) {
    thread::spawn(move || {
        let stream = pal::run_program_with_stream(&program);

        loop {
            let event = match stream.get_event() {
                Some(e) => e,
                None => continue,
            };

            match event {
                Event::Error => (), // TODO: Implement error handling
                Event::Finished => break,
                Event::NeedsInput => {

                }
                Event::Output(ref string) => console_text.write().unwrap().push_str(string)
            };
        }

        println!("finished!");
    });
}
