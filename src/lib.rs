mod env;
mod image;
mod ty;
mod piet_widget;

use piet_widget::PietViewWidget;

use druid::widget::{Scroll, Split, RawLabel, Flex, Label, TextBox, Slider, Button};
use druid::{Color, Size, AppLauncher, Data, Env, LensExt, Lens, Widget, WidgetExt, WindowDesc};
use env::PietEnv;
use wasm_bindgen::prelude::*;

mod macros {
    #[allow(unused_macros)]
    macro_rules! log {
        ( $( $t:tt )* ) => {
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }
    pub(crate) use log;
}


const BACKGROUND: Color = Color::grey8(23);

#[wasm_bindgen]
pub fn wasm_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main()
}

#[derive(Clone, Lens, Data)]
struct AppData {
    env: PietEnv,
    drawing: bool,
}

fn build_root_widget() -> impl Widget<AppData> {
    let visual = Flex::column()
        .with_flex_child(
            PietViewWidget {
                cell_size: Size {
                    width: 0.0,
                    height: 0.0,
                },
            },
            1.0,
        )
        .with_child(
            Flex::column()
                .with_child(
                    Flex::row()
                        .with_flex_child(
                            Button::new("Step")
                                .on_click(|ctx, env: &mut PietEnv, _: &Env| {
                                    env.step();
                                    ctx.request_paint();
                                })
                                .lens(AppData::env)
                                .padding((5., 5.)),
                            1.0,
                        )
                        .padding(8.0),
                )
                .background(BACKGROUND),
        );

    let stack = Label::dynamic(|data, _| format!("{:?}", data))
        .lens(AppData::env.then(PietEnv::stack))
        .expand()
        .padding(5.0);

    Split::columns(visual, stack)
}

pub fn main() {
    let main_window = WindowDesc::new(|| build_root_widget());

    let image = include_bytes!("../hello.png");
    let decoder = png::Decoder::new(&image[..]);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];

    let image = image::PietImg::new(1, info, bytes);
    let mut env = env::PietEnv::new(image);

    // create the initial app state
    let initial_state = AppData {
        env,
        drawing: false,
    };

    // start the application
    AppLauncher::with_window(main_window)
        .launch(initial_state)
        .expect("Failed to launch application");
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::image::PietImg;
    use crate::ty::*;
    use std::fs::File;

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
    }
}
