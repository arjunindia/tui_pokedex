use cursive::theme::{Color, ColorStyle};
use cursive::view::{Nameable, Resizable};
use cursive::views::{Canvas, Dialog, EditView, LinearLayout, ScrollView, TextView};
use cursive::Cursive;
use cursive::Printer;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView};
use reqwest::blocking::Response;
use serde::Deserialize;
use std::io::Cursor;

#[derive(Deserialize)]
struct Species {
    name: String,
}
#[derive(Deserialize)]
struct Type {
    r#type: Species,
}
#[derive(Deserialize)]
struct Sprites {
    front_default: String,
}
// JSON deserialization type for PokeAPI
#[derive(Deserialize)]
struct Ip {
    name: String,
    height: usize,
    weight: usize,
    id: usize,
    species: Species,
    types: Vec<Type>,
    sprites: Sprites,
}
/// Method used to draw the image.
///
/// This takes as input the Canvas state and a printer.
fn draw(_: &(), p: &Printer, img: &DynamicImage) {
    for (x, y, pixel) in img.pixels() {
        // if x % 2 != 0 || y % 2 != 0 {
        //     continue;
        // };
        let style = ColorStyle::new(
            Color::TerminalDefault,
            Color::Rgb(pixel[0], pixel[1], pixel[2]),
        );

        p.with_color(style, |printer| {
            printer.print((x, y), " ");
        });
    }
}

fn trim(img: &mut DynamicImage) {
    let (mut x, mut y) = (0, 0);
    let (mut width, mut height) = img.dimensions();
    // top
    for i in 0..height {
        for j in 0..width {
            let pixel = img.get_pixel(j, i);
            if pixel[3] != 0 {
                y = i;
                break;
            }
        }
        if y != 0 {
            break;
        }
    }
    // bottom
    for i in (0..height).rev() {
        for j in 0..width {
            let pixel = img.get_pixel(j, i);
            if pixel[3] != 0 {
                height = i;
                break;
            }
        }
        if height != img.height() {
            break;
        }
    }
    // left
    for i in 0..width {
        for j in 0..height {
            let pixel = img.get_pixel(i, j);
            if pixel[3] != 0 {
                x = i;
                break;
            }
        }
        if x != 0 {
            break;
        }
    }
    // right
    for i in (0..width).rev() {
        for j in 0..height {
            let pixel = img.get_pixel(i, j);
            if pixel[3] != 0 {
                width = i;
                break;
            }
        }
        if width != img.width() {
            break;
        }
    }
    *img = img.crop_imm(x, y, width - x, height - y).resize_to_fill(
        50,
        50,
        image::imageops::FilterType::Nearest,
    );
}

//handles callback after search now button on the starting dialog
fn search(s: &mut Cursive) {
    // get text from the EditView and format it
    let search = s
        .call_on_name("search", |view: &mut EditView| view.get_content())
        .unwrap()
        .trim()
        .to_ascii_lowercase();

    // URL for pokeapi
    let url = format!("https://pokeapi.co/api/v2/pokemon/{}", &search);

    // using reqwest for client get request
    let Ok(resp) = reqwest::blocking::get(url) 
    else {
        s.add_layer(
            Dialog::text("There was an error connecting to the server!").button("Ok", |s| {
                s.pop_layer();
            }),
        );
        return;
    };
    // Use the type above with serde and reqwest json feature
    let Ok(post_res) = resp.json::<Ip>()
    else {
        s.add_layer(
            Dialog::text("Invalid input! Check if your spelling was correct or the pokemon you entered exists!").button("Ok", |s| {
                s.pop_layer();
            }),
        );
        return;
    };

    let img = reqwest::blocking::get(&post_res.sprites.front_default)
        .unwrap()
        .bytes()
        .unwrap();
    let img = Cursor::new(img);
    let img = ImageReader::new(img).with_guessed_format().expect("msg");
    let mut img = img.decode().expect("Failed to read image");
    trim(&mut img);

    //create comma seperated string of list of pokemon types
    let types = post_res
        .types
        .iter()
        .map(|f| f.r#type.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    // Construct the output text. Might change it to a LinearLayout
    let output_text = format!(
        "Pokedex Entry no:{}\nName:{}\nHeight:{}\nWeight:{}\nSpecies:{}\nTypes:{}",
        post_res.id,
        &post_res.name,
        post_res.height,
        post_res.weight,
        &post_res.species.name,
        types
    );
    let linear_layout = ScrollView::new(
        LinearLayout::vertical()
            .child(
                // Testing Image generation possibility
                Canvas::new(())
                    .with_draw(move |_, p| draw(&(), p, &img))
                    .fixed_size((50, 50)),
            )
            .child(TextView::new(output_text)),
    );
    s.add_layer(
        Dialog::around(linear_layout)
            .title(&post_res.name)
            .button("Go Back", |s| {
                s.pop_layer();
            })
            .fixed_width(50),
    );
}

fn main() {
    let mut siv = cursive::default();

    let linear_layout = LinearLayout::vertical()
        .child(TextView::new(
            "Enter a Pokemon name or Pokedex entry number:",
        ))
        .child(EditView::new().with_name("search"));
    siv.add_layer(
        Dialog::around(linear_layout)
            .title("PokeDex")
            .button("Search Now!", search)
            .button("Quit", |s| (s.quit())),
    );
    siv.run();
}
