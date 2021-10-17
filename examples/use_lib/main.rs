use img::ImageFactory;
use std::error::Error;

mod img;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{:?}", std::env::current_dir());
    let image_factory = ImageFactory::new()?;
    let mut image = image_factory.open_image("data/logo.jpg")?;

    let blurred = image.blur(40.);
    image.mirror();

    image.save("data/mirrored.png")?;
    blurred.save("data/blurred.png")?;
    Ok(())
}
