#[macro_use]
extern crate conrod;
extern crate docopt;
extern crate find_folder;
extern crate pal;
extern crate piston_window;
extern crate rustc_serialize;

use std::thread;
use std::sync::{Arc, Condvar, Mutex, RwLock};

use docopt::Docopt;
use pal::Event;
use piston_window::{AdvancedWindow, EventLoop, OpenGL, PistonWindow, UpdateEvent};

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 1400;

const USAGE: &'static str = "
Paladin: The Pal IDE

Usage:
    paladin --width=<w> --height=<h> --fontsize=<f>
    paladin (-h | --help)
    paladin (-v | --version)
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_width: u32,
    flag_height: u32,
    flag_fontsize: u32,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());

    // Construct the window.
    let mut window: PistonWindow =
        piston_window::WindowSettings::new("Pal IDE", [args.flag_width, args.flag_height])
            .opengl(OpenGL::V3_2)
            .exit_on_esc(true)
            .build()
            .unwrap();
    window.set_ups(60);
    window.set_position([0, 0]);

    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new().build();

    let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let assets = find_folder::Search::KidsThenParents(3, 5)
        .for_folder("inconsolata")
        .expect("Unable to find font folder");
    let font_path = assets.join("Inconsolata-Regular.ttf");
    ui.fonts.insert_from_file(font_path).expect("Unable to insert font file");

    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_texture_cache =
        conrod::backend::piston_window::GlyphCache::new(&mut window, WIDTH, HEIGHT);

    // The image map describing each of our widget->image mappings (in our case, none).
    let image_map = conrod::image::Map::new();

    let mut editor_text = String::new();
    let console_text = Arc::new(RwLock::new(String::new()));
    let mut input_box_text = String::new();
    let inputted_text = Arc::new((Mutex::new(String::new()), Condvar::new()));

    while let Some(event) = window.next() {
        // Convert the piston event to a conrod event.
        if let Some(e) = conrod::backend::piston_window::convert_event(event.clone(), &window) {
            ui.handle_event(e);
        }

        event.update(|_| {
            set_widgets(ui.set_widgets(),
                        &mut editor_text,
                        console_text.clone(),
                        &mut input_box_text,
                        inputted_text.clone(),
                        args.flag_fontsize,
                        &ids)
        });

        window.draw_2d(&event, |c, g| {
            if let Some(primitives) = ui.draw_if_changed() {
                fn texture_from_image<T>(img: &T) -> &T {
                    img
                };
                conrod::backend::piston_window::draw(c,
                                                     g,
                                                     primitives,
                                                     &mut text_texture_cache,
                                                     &image_map,
                                                     texture_from_image);
            }
        });
    }
}

widget_ids! {
    struct Ids {
        canvas,
        run_button,
        editor_color,
        editor,
        editor_scroll,
        separator,
        console_color,
        console,
        input_color,
        input
    }
}



fn set_widgets(ref mut ui: conrod::UiCell,
               editor_text: &mut String,
               console_text: Arc<RwLock<String>>,
               input_box_text: &mut String,
               inputted_text: Arc<(Mutex<String>, Condvar)>,
               font_size: u32,
               ids: &Ids) {
    use conrod::{color, widget, Colorable, Labelable, Positionable, Sizeable, Widget};

    widget::Canvas::new().color(color::DARK_CHARCOAL).scroll_kids().set(ids.canvas, ui);

    let dim = ui.window_dim();
    let canvas_width = dim[0] - 150.0;
    let canvas_height = dim[1];
    let editor_color_height = canvas_height * 9.0 / 14.0;
    let margin = 10.0;
    let width_padding = 5.0;
    let height_padding = 2.0;
    let line_spacing = font_size as f64 * 1.2;
    let button_width = 120.0;
    let button_height = canvas_height / 35.0;
    let button_font_size = button_height as u32 - 2;
    let separator_offset = 10.0;
    let separator_height = canvas_height * 3.0 / 56.0;
    let console_color_height =  canvas_height / 4.0;
    let input_height = canvas_height / 35.0;

    widget::Rectangle::fill_with([canvas_width, editor_color_height], color::BLACK)
        .top_left_with_margin_on(ids.canvas, margin)
        .set(ids.editor_color, ui);

    for edit in widget::TextEdit::new(editor_text)
        .color(color::WHITE)
        .padded_w_of(ids.editor_color, width_padding)
        .padded_h_of(ids.editor_color, height_padding)
        .x_y_relative_to(ids.editor_color, 0.0, 0.0)
        .align_text_left()
        .line_spacing(line_spacing)
        .font_size(font_size)
        .set(ids.editor, ui) {
        *editor_text = edit;
    }

    if widget::Button::new()
        .top_right_with_margin_on(ids.canvas, margin)
        .w_h(button_width, button_height)
        .label("Run code")
        .label_font_size(button_font_size)
        .set(ids.run_button, ui)
        .into_iter()
        .was_clicked() {
        console_text.write().unwrap().clear();
        run_program(editor_text.clone(),
                    console_text.clone(),
                    inputted_text.clone());
    }


    widget::Rectangle::fill_with([canvas_width, separator_height], color::DARK_BLUE)
        .down_from(ids.editor_color, separator_offset)
        .set(ids.separator, ui);

    widget::Rectangle::fill_with([canvas_width, console_color_height], color::BLACK)
        .down_from(ids.separator, separator_offset)
        .set(ids.console_color, ui);

    widget::Text::new(&*console_text.read().unwrap())
        .color(color::WHITE)
        .padded_w_of(ids.console_color, width_padding)
        .padded_h_of(ids.console_color, height_padding)
        .x_y_relative_to(ids.console_color, 0.0, 0.0)
        .align_text_left()
        .line_spacing(line_spacing)
        .font_size(font_size)
        .set(ids.console, ui);

    for edit in widget::TextBox::new(input_box_text)
        .w_h(canvas_width, input_height)
        .down_from(ids.console_color, separator_offset)
        .font_size(font_size)
        .set(ids.input, ui) {
        match edit {
            widget::text_box::Event::Update(new_str) => *input_box_text = new_str,
            widget::text_box::Event::Enter => {
                let &(ref string, ref condvar) = &*inputted_text;
                let mut unlocked_string = string.lock().unwrap();

                *unlocked_string = input_box_text.clone();
                input_box_text.clear();
                condvar.notify_one();
            }
        }
    }
}

fn run_program(program: String,
               console_text: Arc<RwLock<String>>,
               inputted_text: Arc<(Mutex<String>, Condvar)>) {
    thread::spawn(move || {
        println!("running program...");

        let stream = pal::run_program_with_stream(&program);

        loop {
            let event = match stream.get_event() {
                Some(e) => e,
                None => continue,
            };

            println!("got event: {:?}", event);

            match event {
                Event::Error => (), // TODO: Implement error handling
                Event::Finished => break,
                Event::NeedsInput => {
                    let &(ref string, ref condvar) = &*inputted_text;
                    let mut guard = string.lock().unwrap();

                    guard = condvar.wait(guard).unwrap();
                    stream.write_input(&guard);

                    guard.clear();
                }
                Event::Output(ref string) => console_text.write().unwrap().push_str(string),
            };
        }

        println!("finished!\n");
    });
}
