#![windows_subsystem = "windows"] 
#![cfg(windows)]

extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;
use nwd::NwgUi;
use nwg::NativeUi;

extern crate image as img;
use img::imageops::colorops::grayscale;

use std::fs;
use std::path::Path;
use std::io::prelude::*;
use std::{thread, time};

use std::env;

//todo: delete old textinput element when new image being displayed. also make video work. 

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(size: (1000, 800), position: (500, 500), title: "Ascii Player", flags: "WINDOW|VISIBLE")]
    window: nwg::Window,

    #[nwg_layout(parent: window, max_column: Some(14), max_row: Some(11))]
    layout: nwg::GridLayout,

    #[nwg_resource(title: "Save File", action: nwg::FileDialogAction::Save)]
    dialog_save: nwg::FileDialog,

    #[nwg_resource(title: "Open File", action: nwg::FileDialogAction::Open, filters: "Ascii Greyscale Image and Video Files (*.agif;*.agvf)")]
    dialog_display: nwg::FileDialog,
    #[nwg_control(text: "Display", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, col_span: 7, row_span: 2)]
    #[nwg_events(OnButtonClick: [App::open_file_display])]
    open_btn: nwg::Button,

    #[nwg_resource(title: "Open File", action: nwg::FileDialogAction::Open, filters: "Png and Jpeg Files (*.png;*.jpg;*.jpeg)")]
    dialog_convert: nwg::FileDialog,
    #[nwg_control(text: "Convert", focus: true)]
    #[nwg_layout_item(layout: layout, col: 7, row: 0, col_span: 7, row_span: 2)]
    #[nwg_events(OnButtonClick: [App::open_file_convert])]
    open_btn_convert: nwg::Button,

    #[nwg_control(text: "Title:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 1)]
    title_label: nwg::Label,
    #[nwg_control(text:"")]
    #[nwg_layout_item(layout: layout, col: 1, row: 2, col_span: 2)]
    title: nwg::TextInput,

    #[nwg_control(text: "Author:")]
    #[nwg_layout_item(layout: layout, col: 3, row: 2, col_span: 1)]
    author_label: nwg::Label,
    #[nwg_control(text:"")]
    #[nwg_layout_item(layout: layout, col: 4, row: 2, col_span: 2)]
    author: nwg::TextInput,

    #[nwg_control(text: "Time:")]
    #[nwg_layout_item(layout: layout, col: 6, row: 2, col_span: 1)]
    timestamp_label: nwg::Label,
    #[nwg_control(text:"")]
    #[nwg_layout_item(layout: layout, col: 7, row: 2, col_span: 2)]
    timestamp: nwg::TextInput,

    #[nwg_control(text: "", focus: true, readonly: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 3, col_span: 14, row_span: 8)]
    frame: nwg::TextBox,
}

impl App {
    //todo: change font size depending on dimensions of image, support video, (maybe do a save dialog for convert), ship
    fn open_file_convert(&self) {
        if self.dialog_convert.run(Some(&self.window)) {
            //metadata format: title,author,timestamp
            let selected_item = self.dialog_convert.get_selected_item().unwrap();
            //selected_item.unwrap().into_string().unwrap() is the file path
            let selected_item = selected_item.into_string().unwrap();
            let file_extension = Path::new(&selected_item).extension().unwrap().to_str().unwrap();
            if file_extension == "png" || file_extension == "jpeg" || file_extension == "jpg" {
                let image = img::open(&Path::new(&selected_item)).unwrap();
                let image = grayscale(&image);
                let (length, height) = image.dimensions();
                let mut new_format = String::new();
                for y in 0..height {
                    new_format.push_str("\n");
                    for x in 0..length {
                        let pixel = image[(x,y)];
                        if pixel.0[0] < 85 {
                            new_format.push('░');
                        } else if pixel.0[0] < 171 {
                            new_format.push('▒');
                        } else {
                            new_format.push('▓');
                        }
                    }
                }
                self.frame.set_text(&new_format);
                if self.dialog_save.run(Some(&self.window)) {
                    let save_path = self.dialog_save.get_selected_item().unwrap().into_string().unwrap()+".agif";
                    let mut save_file = fs::File::create(Path::new(&save_path)).unwrap();
                    save_file.write_all(new_format.as_bytes()).unwrap();
                }
            } 
        }
    }
    fn open_file_display(&self) {
        //IMPORTANT: make sure the file is valid
        if self.dialog_display.run(Some(&self.window)) {
            let selected_item = self.dialog_display.get_selected_item().unwrap();
            //selected_item.unwrap().into_string().unwrap() is the file path
            let selected_item = selected_item.into_string().unwrap();
            let file_extension = Path::new(&selected_item).extension().unwrap().to_str().unwrap();
            //TODO: check if file extension is valid
            let file_contents = fs::read_to_string(&selected_item).expect("Something went wrong reading the file");
            //turn file lines into a vec from an iterator
            let mut file_lines: Vec<&str> = file_contents.lines().collect();
            //get metadata (first line)
            let metadata = file_lines[0];
            //split line and get list of metadata information 
            let metadata: Vec<&str> = metadata.split("|").collect();
            //remove metadata, prepare file_lines for parsing and display
            file_lines.remove(0);
            //check if its an image for video
            //let mut display_box = &self.display_box;
            //let mut display_box = Default::default();
            if file_extension == "agif" {
                let image = file_lines.join("\r\n");
                //display image with small font? disable copy paste
                //iterate through metadata and display relevant info
                //nwg::TextBox::builder().size((100,100)).readonly(true).text(&image).parent(&self.window).build(&mut display_box).expect("Failed to build textbox");
                //self.layout.add_child(0, 1, &display_box);
                self.frame.set_text(&image);
                self.title.set_text(metadata[0]);
                self.title.set_readonly(true);
                self.author.set_text(metadata[1]);
                self.author.set_readonly(true);
                self.timestamp.set_text(metadata[2]);
                self.timestamp.set_readonly(true);
            }
            else if file_extension == "agvf" {
                let video = file_lines.join("\r\n");
                //metadata format: title,author,timestamp,fps
                let fps: u64 = metadata[2].parse().unwrap();
                let wait_duration = 1000/fps;
                let frames: Vec<&str> = video.split("===").collect();
                for frame in frames {
                    thread::sleep(time::Duration::from_millis(wait_duration));
                    self.frame.set_text(&frame);
                }
            }
        }
    }

    fn check_for_args(&self) {
        let args: Vec<String> = env::args().collect();
        if args.len() == 1 {
          let file_name = &args[1];
          //open the file
          let file_extension = Path::new(&file_name).extension().unwrap().to_str().unwrap();
          let file_contents = fs::read_to_string(&file_name).expect("Something went wrong reading the file");
          let mut file_lines: Vec<&str> = file_contents.lines().collect();
          let metadata = file_lines[0];
          let metadata: Vec<&str> = metadata.split("|").collect();
          file_lines.remove(0);
          if file_extension == "agif" {
              let image = file_lines.join("\r\n");
              //display image with small font? disable copy paste
              //iterate through metadata and display relevant info
              //nwg::TextBox::builder().size((100,100)).readonly(true).text(&image).parent(&self.window).build(&mut display_box).expect("Failed to build textbox");
              //self.layout.add_child(0, 1, &display_box);
              self.frame.set_text(&image);
              self.title.set_text(metadata[0]);
              self.title.set_readonly(true);
              self.author.set_text(metadata[1]);
              self.author.set_readonly(true);
              self.timestamp.set_text(metadata[2]);
              self.timestamp.set_readonly(true);
          }
          else if file_extension == "agvf" {
              let video = file_lines.join("\r\n");
              //metadata format: title,author,timestamp,fps
              let fps: u64 = metadata[2].parse().unwrap();
              let wait_duration = 1000/fps;
              let frames: Vec<&str> = video.split("===").collect();
              for frame in frames {
                  thread::sleep(time::Duration::from_millis(wait_duration));
                  self.frame.set_text(&frame);
              }
          }
        }
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let _app = App::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}