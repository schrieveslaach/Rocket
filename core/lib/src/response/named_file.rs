use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use headers::{Header as HeaderTrait, HeaderValue, IfModifiedSince};
use tokio::fs::File;

use crate::http::{ContentType, Header, Status};
use crate::request::Request;
use crate::response::{self, Responder};
use crate::Response;

/// A file with an associated name; responds with the Content-Type based on the
/// file extension.
#[derive(Debug)]
pub struct NamedFile {
    path: PathBuf,
    file: File,
    modified: Option<SystemTime>,
}

impl NamedFile {
    /// Attempts to open a file in read-only mode.
    ///
    /// # Errors
    ///
    /// This function will return an error if path does not already exist. Other
    /// errors may also be returned according to
    /// [`OpenOptions::open()`](std::fs::OpenOptions::open()).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use rocket::get;
    /// use rocket::response::NamedFile;
    ///
    /// #[get("/")]
    /// async fn index() -> Option<NamedFile> {
    ///     NamedFile::open("index.html").await.ok()
    /// }
    /// ```
    pub async fn open<P: AsRef<Path>>(path: P) -> io::Result<NamedFile> {
        // FIXME: Grab the file size here and prohibit `seek`ing later (or else
        // the file's effective size may change), to save on the cost of doing
        // all of those `seek`s to determine the file size. But, what happens if
        // the file gets changed between now and then?
        let file = File::open(path.as_ref()).await?;
        Ok(NamedFile {
            path: path.as_ref().to_path_buf(),
            file,
            modified: None,
        })
    }

    /// Attempts to open a file in the same manner as `NamedFile::open` and
    /// reads the modification timestamp of the file that will be used to
    /// respond with the `Last-Modified` header. This enables HTTP caching by
    /// comparing the modification timestamp with the `If-Modified-Since`
    /// header when requesting the file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rocket::response::NamedFile;
    ///
    /// # #[allow(unused_variables)]
    /// # rocket::async_test(async {
    /// let file = NamedFile::with_last_modified_date("foo.txt").await;
    /// # });
    /// ```
    pub async fn with_last_modified_date<P: AsRef<Path>>(path: P) -> io::Result<NamedFile> {
        let mut named_file = NamedFile::open(path).await?;
        named_file.modified = named_file.metadata().await?.modified().ok();
        Ok(named_file)
    }

    /// Retrieve the underlying `File`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::response::NamedFile;
    ///
    /// # async fn f() -> std::io::Result<()> {
    /// let named_file = NamedFile::open("index.html").await?;
    /// let file = named_file.file();
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn file(&self) -> &File {
        &self.file
    }

    /// Retrieve a mutable borrow to the underlying `File`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::response::NamedFile;
    ///
    /// # async fn f() -> std::io::Result<()> {
    /// let mut named_file = NamedFile::open("index.html").await?;
    /// let file = named_file.file_mut();
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Take the underlying `File`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rocket::response::NamedFile;
    ///
    /// # async fn f() -> std::io::Result<()> {
    /// let named_file = NamedFile::open("index.html").await?;
    /// let file = named_file.take_file();
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn take_file(self) -> File {
        self.file
    }

    /// Retrieve the path of this file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rocket::response::NamedFile;
    ///
    /// # async fn demo_path() -> std::io::Result<()> {
    /// let file = NamedFile::open("foo.txt").await?;
    /// assert_eq!(file.path().as_os_str(), "foo.txt");
    /// # Ok(())
    /// # }
    /// ```
    #[inline(always)]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}

/// Streams the named file to the client. Sets or overrides the Content-Type in
/// the response according to the file's extension if the extension is
/// recognized. See [`ContentType::from_extension()`] for more information. If
/// you would like to stream a file with a different Content-Type than that
/// implied by its extension, use a [`File`] directly.
impl<'r> Responder<'r, 'static> for NamedFile {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        if let Some(last_modified) = &self.modified {
            if let Some(if_modified_since) = req.headers().get_one("If-Modified-Since") {
                if let Ok(if_modified_since) = parse_if_modified_since(if_modified_since) {
                    if !if_modified_since.is_modified(*last_modified) {
                        return Response::build().status(Status::NotModified).ok();
                    }
                }
            }
        }

        let mut response = self.file.respond_to(req)?;
        if let Some(ext) = self.path.extension() {
            if let Some(ct) = ContentType::from_extension(&ext.to_string_lossy()) {
                response.set_header(ct);
            }
        }

        if let Some(last_modified) = self.modified.map(|m| IfModifiedSince::from(m)) {
            let mut headers = Vec::with_capacity(1);
            last_modified.encode(&mut headers);
            let v = headers[0].to_str().unwrap();
            response.set_header(Header::new("Last-Modified", v.to_string()));
        }

        Ok(response)
    }
}

fn parse_if_modified_since(header: &str) -> Result<IfModifiedSince, String> {
    let headers = vec![HeaderValue::from_str(header).map_err(|e| e.to_string())?];
    let mut headers_it = headers.iter();
    Ok(IfModifiedSince::decode(&mut headers_it).map_err(|e| e.to_string())?)
}

impl Deref for NamedFile {
    type Target = File;

    fn deref(&self) -> &File {
        &self.file
    }
}

impl DerefMut for NamedFile {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.file
    }
}
