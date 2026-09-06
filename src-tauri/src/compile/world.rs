//! The `typst::World` implementation backing ams' compile engine.
//!
//! The main document's text lives in memory (`main_source`) and is updated
//! directly via [`Source::replace`] on each edit, since it may not (yet) be
//! saved to disk. Everything else a document references — images,
//! `#bibliography(...)` files, etc. — is resolved from the filesystem,
//! relative to the document's directory, matching the Typst CLI.

use std::path::PathBuf;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{DiagSpan, DiagSpanKind, FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt, World, WorldExt};
use typst_kit::datetime::Time;
use typst_kit::downloader::SystemDownloader;
use typst_kit::files::{FileLoader, FileStore, FsRoot};
use typst_kit::fonts::FontStore;
use typst_kit::packages::SystemPackages;

use super::fonts::default_fonts;

/// Loads sibling files (images, `.bib` files, ...) relative to the document's
/// directory, and resolves `#import "@preview/..."` packages.
///
/// Packages come from the same places the Typst CLI looks — the system data
/// directory, then the local cache, then Typst Universe — so a package already
/// downloaded by the CLI is reused, and anything fetched here is cached for
/// next time. Only the first use of a package needs the network.
struct DocFileLoader {
    project: FsRoot,
    packages: SystemPackages,
}

impl FileLoader for DocFileLoader {
    fn load(&self, id: FileId) -> FileResult<Bytes> {
        match id.root() {
            VirtualRoot::Project => self.project.load(id.vpath()),
            VirtualRoot::Package(spec) => {
                let root = self.packages.obtain(spec).map_err(FileError::Package)?;
                root.load(id.vpath())
            }
        }
    }
}

/// A reusable Typst compilation environment for a single document.
///
/// Keep one `AmsWorld` alive for the lifetime of an open document (see
/// [`crate::compile::session::CompileSession`]) rather than rebuilding it per
/// keystroke, so Typst's own memoization (`comemo`) can skip re-evaluating
/// unchanged parts of the document on each recompile.
pub struct AmsWorld {
    library: LazyHash<Library>,
    fonts: FontStore,
    files: FileStore<DocFileLoader>,
    main_id: FileId,
    main_source: Source,
    time: Time,
}

impl AmsWorld {
    /// Creates a world for a document whose sibling assets resolve relative
    /// to `root` (typically the document's parent directory), with `text` as
    /// the initial content of the main file named `file_name`.
    pub fn new(root: PathBuf, file_name: &str, text: impl Into<String>) -> Self {
        let main_id = RootedPath::new(
            VirtualRoot::Project,
            VirtualPath::new(file_name).expect("file name is a valid virtual path"),
        )
        .intern();

        Self {
            library: LazyHash::new(Library::default()),
            fonts: default_fonts(),
            files: FileStore::new(DocFileLoader {
                project: FsRoot::new(root),
                packages: SystemPackages::new(SystemDownloader::new(format!(
                    "ams/{}",
                    env!("CARGO_PKG_VERSION")
                ))),
            }),
            main_id,
            main_source: Source::new(main_id, text.into()),
            time: Time::system(),
        }
    }

    /// Creates a world with no filesystem backing, for tests/documents that
    /// reference no sibling files.
    pub fn detached(text: impl Into<String>) -> Self {
        Self::new(PathBuf::from("."), "main.typ", text)
    }

    /// Replaces the main document's text, reusing the previous syntax tree
    /// where possible (an incremental reparse of just the changed range).
    pub fn set_text(&mut self, text: &str) {
        self.main_source.replace(text);
    }

    /// The file id of the document being edited.
    pub fn main_id(&self) -> FileId {
        self.main_id
    }

    /// The document's current source, for tooling that inspects it directly
    /// (completions) rather than going through the `World` trait.
    pub fn main_source(&self) -> &Source {
        &self.main_source
    }

    /// 1-based line and column of a diagnostic inside the main document.
    ///
    /// Returns `None` for diagnostics that don't point anywhere (Typst calls
    /// these detached) or that point into a sibling file rather than the
    /// document being edited — the editor can only usefully jump to its own
    /// buffer.
    pub fn location_of(&self, span: DiagSpan) -> Option<(usize, usize)> {
        let id = match span.get() {
            DiagSpanKind::Detached => return None,
            DiagSpanKind::Number { id, .. } | DiagSpanKind::Range { id, .. } => id,
        };
        if id != self.main_id {
            return None;
        }

        let range = self.range(span)?;
        let (line, column) = self.main_source.lines().byte_to_line_column(range.start)?;
        Some((line + 1, column + 1))
    }

    /// Prepares this world for a fresh compile: marks cached sibling files
    /// stale (so an on-disk change is picked up, while still reusing the
    /// old content if it didn't change) and lets `today()` be refetched.
    ///
    /// Safe to call before every compile, including the first.
    pub fn begin_compile(&mut self) {
        self.files.reset();
        self.time.reset();
    }
}

impl World for AmsWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.main_id
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main_id {
            Ok(self.main_source.clone())
        } else {
            self.files.source(id)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == self.main_id {
            Ok(Bytes::from_string(self.main_source.clone()))
        } else {
            self.files.file(id)
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}
