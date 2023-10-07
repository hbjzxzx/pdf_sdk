use win_test::run;
// use winit::{
//     event::{Event, WindowEvent},
//     event_loop::{EventLoop},
//     window::{WindowBuilder},
// };

use std::env::current_dir;

fn main() {
    println!("current dir is: {}", current_dir().unwrap().to_str().unwrap());
    run()
}
