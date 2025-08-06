use hdf5::File;
use std::error::Error;
use std::{env, fs};
use tiff::encoder;

#[derive(Debug)]
pub struct Config {
    // Config will be called to read arguments from the command line
    pub file_name: String,
    pub output_dir: String,
    pub res_levels: usize,
    pub channels: usize,
}

impl Config {
    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next(); // discard the program name

        let file_name = match args.next() {
            Some(arg) => arg,
            None => return Err("No .ims file defined"),
        };
        let output_dir = match args.next() {
            Some(arg) => arg,
            None => return Err("No output directory defined"),
        };
        let res_levels = match args.next() {
            Some(arg) => arg
                .parse::<usize>()
                .expect("Resolution levels should be an integer"),
            None => return Err("Missing number of resolution levels"),
        };
        let channels = match args.next() {
            Some(arg) => arg
                .parse::<usize>()
                .expect("Number of channels should be an integer"),
            None => return Err("Missing number of channels"),
        };

        Ok(Config {
            file_name,
            output_dir,
            res_levels,
            channels,
        })
    }
}

pub struct Image {}
impl Image {
    pub fn new() -> Image {
        Image {}
    }
}

pub fn convert(conf: Config) -> Result<(), Box<dyn Error>> {
    let file_name = &conf.file_name;
    for i in 0..conf.res_levels {
        for j in 0..conf.channels {
            let img = read_h5(file_name, i, j);
        }
    }
    Ok(())
}

pub fn read_h5(file_name: &str, res: usize, chan: usize) -> Result<Image, Box<dyn Error>> {
    let h5f = File::open(file_name)?;
    let data = h5f.dataset(&format!(
        "DataSet/ResolutionLevel {res}/TimePoint 0/Channel {chan}/Data"
    ))?;
    let ds_slice: Vec<usize> = data.read_raw()?;
    println!("{:?}", ds_slice);
    Ok(Image {})
}
