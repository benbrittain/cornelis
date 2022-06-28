use num_derive::FromPrimitive;
use png::OutputInfo;
use std::{collections::VecDeque, fs::File};

mod env;
mod image;
mod ty;

fn main() -> Result<(), std::io::Error> {
    let decoder = png::Decoder::new(File::open("hello.png")?);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let bytes = &buf[..info.buffer_size()];

    // TODO verify more things about the PNG
    assert!(info.color_type == png::ColorType::Rgb);

    let image = image::PietImg::new(1, info, bytes);
    let mut env = env::PietEnv::new(&image);

    env.step();

    Ok(())
}

#[test]
fn assert_color_decode_in_one_codel_golden_image() {
    let decoder = png::Decoder::new(File::open("hello.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let image = PietImg::new(1, info, bytes);

    assert_eq!(PietColor::from(&image[Codel::new(0, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[Codel::new(1, 0)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[Codel::new(0, 1)]), PietColor::Red);
    assert_eq!(PietColor::from(&image[Codel::new(10, 0)]), PietColor::Red);
    assert_eq!(
        PietColor::from(&image[Codel::new(11, 0)]),
        PietColor::DarkRed
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(18, 0)]),
        PietColor::Magenta
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(19, 0)]),
        PietColor::DarkMagenta
    );
    assert_eq!(PietColor::from(&image[Codel::new(20, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[Codel::new(21, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[Codel::new(27, 0)]), PietColor::Blue);
    assert_eq!(PietColor::from(&image[Codel::new(29, 0)]), PietColor::Blue);
    assert_eq!(
        PietColor::from(&image[Codel::new(11, 1)]),
        PietColor::Magenta
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(19, 10)]),
        PietColor::Black
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(29, 24)]),
        PietColor::Green
    );
    assert_eq!(
        PietColor::from(&image[Codel::new(29, 28)]),
        PietColor::LightYellow
    );
}

#[test]
fn flood_fill_test_in_one_codel_golden_image() {
    let decoder = png::Decoder::new(File::open("hello.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    let image = PietImg::new(1, info, bytes);

    // the three vertical magenta blocks on the top row
    let flood_fill = image.get_codels_in_block(Codel::new(19, 0));
    assert_eq!(flood_fill.codels.len(), 3);

    // the 4 pixel pyramid inside the main red start region
    let flood_fill = image.get_codels_in_block(Codel::new(4, 8));
    assert_eq!(flood_fill.codels.len(), 4);

    // a black singleton cube
    let flood_fill = image.get_codels_in_block(Codel::new(4, 6));
    assert_eq!(flood_fill.codels.len(), 1);

    let flood_fill = image.get_codels_in_block(Codel::new(25, 25));
}
