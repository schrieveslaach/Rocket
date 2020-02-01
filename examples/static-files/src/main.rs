#[cfg(test)] mod tests;

use rocket_contrib::serve::{StaticFiles, crate_relative};

// If we wanted or needed to serve files manually, we'd use `NamedFile`. Always
// prefer to use `StaticFiles`!
mod manual {
    use std::path::{PathBuf, Path};
    use rocket::response::NamedFile;

    #[rocket::get("/second/<path..>")]
    pub async fn second(path: PathBuf) -> Option<NamedFile> {
        let mut path = Path::new(super::crate_relative!("static")).join(path);
        if path.is_dir() {
            path.push("index.html");
        }

        NamedFile::open(path).await.ok()
    }

    #[rocket::get("/with-caching/rocket-icon.jpg")]
    pub async fn cached_icon() -> Option<NamedFile> {
        NamedFile::with_last_modified_date("static/rocket-icon.jpg").await.ok()
    }

}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", rocket::routes![manual::second, manual::cached_icon])
        .mount("/", StaticFiles::from(crate_relative!("static")))
}
