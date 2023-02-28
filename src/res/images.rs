use {
    rand::{seq::SliceRandom, thread_rng},
    serenity::{builder::CreateMessage, prelude::TypeMapKey, utils::Color},
    std::{
        env::args,
        fs::read_dir,
        path::{Path, PathBuf},
    },
};

const COLOR: Color = Color::from_rgb(0, 0, 0);

pub struct Images;

impl TypeMapKey for Images {
    type Value = Vec<Image>;
}

impl Images {
    pub fn get_image_files() -> Result<<Self as TypeMapKey>::Value, String> {
        let mut args = args();

        let dir = args.nth(1).ok_or("Missing directory argument")?;
        let title = args.next().get_or_insert("Kun".to_string()).clone();

        let read_dir = read_dir(dir).map_err(|e| e.to_string())?;
        let filtered = read_dir
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        const EXTENSIONS: [&'static str; 3] = ["jpg", "jpeg", "png"];

        let images = filtered.into_iter().enumerate().filter_map(|(id, de)| {
            let path = de.path();
            let file_name = path.file_name()?.to_str()?.to_string();
            let title = title.clone();

            EXTENSIONS
                .contains(&path.extension()?.to_str()?)
                .then_some(Image {
                    file_name,
                    title,
                    path,
                    id,
                })
        });
        Ok(images.collect())
    }

    pub fn choose(images: &<Self as TypeMapKey>::Value) -> Option<&Image> {
        images.choose(&mut thread_rng())
    }
}

#[derive(Debug)]
pub struct Image {
    file_name: String,
    title: String,
    path: PathBuf,
    id: usize,
}

impl Image {
    fn file_name(&self) -> &str {
        &self.file_name
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn id(&self) -> usize {
        self.id
    }

    pub fn as_embed<'a, 'b>(&'b self, m: &'a mut CreateMessage<'b>) -> &'a mut CreateMessage<'b> {
        m.embed(|e| {
            e.title(self.title())
                .attachment(self.file_name())
                .color(COLOR)
                .footer(|f| f.text(self.id()))
        })
        .add_file(self.path())
    }
}
