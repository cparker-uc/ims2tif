use hdf5::{File, Result};
use ndarray::{Array3, s};
use std::error::Error;
use std::fs;
// use std::io::{self, Write};
use std::path::Path;
use tiff::encoder::{TiffEncoder, TiffKindBig, colortype::Gray8};
// use tiff::tags::CompressionMethod;

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

/// This struct will implement Iterator, so that we can iterate over the h5 file and yield one
/// chunk at a time. This will be done in the next function (required to implement Iterator) and
/// it will return Option<Array3<usize>>. I think this needs to be associated with the actual h5 file
/// so that we have info in self that can let us call read_h5
pub struct ImageSlicer<'a> {
    current: usize,
    slice_size: usize,
    res: usize,
    chan: usize,
    config: &'a Config,
}

impl<'a> ImageSlicer<'a> {
    pub fn new(slice_size: usize, res: usize, chan: usize, config: &'a Config) -> Self {
        ImageSlicer {
            current: 0,
            slice_size,
            res,
            chan,
            config,
        }
    }

    /// Read the next slice
    fn read_h5(&self) -> Result<Array3<u8>, Box<dyn Error>> {
        let h5f = File::open(&self.config.file_name)?;
        let data = h5f.dataset(&format!(
            "DataSet/ResolutionLevel {}/TimePoint 0/Channel {}/Data",
            self.res, self.chan,
        ))?;
        // Check the size of the dataset
        let ds_size = data.shape();
        let (nx, ny, nz) = (ds_size[0], ds_size[1], ds_size[2]);
        // Check whether we are closer to nz than slice_size indices
        let dz = self.slice_size.min(nz - self.current);
        let dz = self.current + dz;

        let slice: Array3<u8> =
            data.read_slice::<u8, _, ndarray::Dim<[usize; 3]>>((0..nx, 0..ny, self.current..dz))?;

        Ok(slice)
    }
}

impl<'a> Iterator for ImageSlicer<'a> {
    type Item = Array3<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        // Read the next slice, and return None to signal the iterator has run its course if we
        // error while reading
        let slice: Array3<u8> = match ImageSlicer::read_h5(&self) {
            Ok(arr) => arr,
            Err(_) => return None,
        };
        // Increment our current position in the array
        self.current += self.slice_size;
        Some(slice)
    }
}

/// This function contains the real logic of the program
/// It uses the Config struct to loop through the number of res_levels and channels
/// And for each, it slices the dataset if it's too large and
pub fn convert(conf: Config) -> Result<(), Box<dyn Error>> {
    let out_path = Path::new(&conf.output_dir);
    if out_path.exists() == false {
        fs::create_dir_all(out_path)?;
    }
    let filename_root = Path::new(&conf.file_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Couldn't get filename root from the ims file.");
    for res in 0..7 {
        for chan in 0..2 {
            // Create a tiff file for this slicer
            let out_file = format!("{filename_root}_Res{res}_Chan{chan}.tif");
            let out_path_ = out_path.join(out_file);
            let file = fs::File::create(out_path_)?;
            let mut tiff_file = TiffEncoder::new_big(file).unwrap();

            // Instantiate a slicer and loop through
            let slicer = ImageSlicer::new(32, res, chan, &conf);

            for (idx, slice) in slicer.enumerate() {
                println!("ResolutionLevel {res}, Channel {chan}, Slice number {idx}");
                // io::stdout().flush().unwrap();
                // Write to tif file
                write_tiff(slice, &mut tiff_file)?;
            }
        }
    }
    Ok(())
}

fn write_tiff(
    slice: Array3<u8>,
    tiff_file: &mut TiffEncoder<fs::File, TiffKindBig>,
) -> Result<(), Box<dyn Error>> {
    let slice_size = slice.shape();
    let (nx, ny, nz) = (slice_size[0], slice_size[1], slice_size[2]);
    for z_ in 0..nz {
        let image = tiff_file.new_image::<Gray8>(nx as u32, ny as u32)?;
        let frame = slice.slice(s![.., .., z_]).to_owned();
        let flattened_data = frame
            .as_slice()
            .expect("Had an issue while flattening the array to write a frame");
        image.write_data(flattened_data)?;
    }
    Ok(())
}
