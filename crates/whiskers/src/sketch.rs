use kurbo::Affine;
use vsvg::path::IntoBezPathTolerance;
use vsvg::{
    Document, DocumentTrait, LayerID, PageSize, Path, PathMetadata, Transforms, DEFAULT_TOLERANCE,
};

pub struct Sketch {
    document: Document,
    transform_stack: Vec<Affine>,
    target_layer: LayerID,
    tolerance: f64,
    path_metadata: PathMetadata,
}

impl Default for Sketch {
    fn default() -> Self {
        Self::new()
    }
}
impl Sketch {
    pub fn new() -> Self {
        Self::with_document(Document::default())
    }

    pub fn with_document(mut document: Document) -> Self {
        let target_layer = 0;
        document.ensure_exists(target_layer);

        Self {
            document,
            tolerance: DEFAULT_TOLERANCE,
            transform_stack: vec![Affine::default()],
            target_layer,
            path_metadata: PathMetadata::default(),
        }
    }

    pub fn set_layer(&mut self, layer_id: LayerID) -> &mut Self {
        self.document.ensure_exists(layer_id);
        self.target_layer = layer_id;
        self
    }

    /// Returns the sketch's width in pixels.
    ///
    /// If the page size is not set, it defaults to 400px.
    pub fn width(&self) -> f64 {
        self.document
            .metadata()
            .page_size
            .map(|p| p.w())
            .unwrap_or(400.0)
    }

    /// Returns the sketch's height in pixels.
    ///
    /// If the page size is not set, it defaults to 400px.
    pub fn height(&self) -> f64 {
        self.document
            .metadata()
            .page_size
            .map(|p| p.h())
            .unwrap_or(400.0)
    }

    pub fn page_size(&mut self, page_size: PageSize) -> &mut Self {
        self.document.metadata_mut().page_size = Some(page_size);
        self
    }

    pub fn color(&mut self, color: impl Into<vsvg::Color>) -> &mut Self {
        self.path_metadata.color = color.into();
        self
    }

    pub fn stroke_width(&mut self, width: impl Into<f64>) -> &mut Self {
        self.path_metadata.stroke_width = width.into();
        self
    }

    /// Push the current matrix onto the stack.
    ///
    /// A copy of the current transform matrix is pushed onto the stack. Use this before applying
    /// temporary transforms that you want to revert later with [`pop_matrix`].
    pub fn push_matrix(&mut self) -> &mut Self {
        self.transform_stack
            .push(self.transform_stack.last().copied().unwrap_or_default());
        self
    }

    /// Push the identity matrix onto the stack.
    ///
    /// Use this if you want to temporarily reset the transform matrix and later revert to the
    /// current matrix with [`pop_matrix`].
    pub fn push_matrix_reset(&mut self) -> &mut Self {
        self.transform_stack.push(Affine::default());
        self
    }

    /// Pop the current transform matrix from the stack, restoring the previously pushed matrix.
    pub fn pop_matrix(&mut self) -> &mut Self {
        if self.transform_stack.len() == 1 {
            log::warn!("pop_matrix: stack underflow");
            return self;
        }

        self.transform_stack.pop();
        self
    }

    /// Push the current matrix onto the stack, apply the given function, then pop the matrix.
    ///
    /// This is a convenience method for draw code that require a temporary change of the current
    /// transform matrix.
    pub fn push_matrix_and(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        self.push_matrix();
        f(self);
        self.pop_matrix();
        self
    }

    pub fn center(&mut self) -> &mut Self {
        self.document_mut().center_content();
        self
    }

    pub fn document(&self) -> &Document {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut Document {
        &mut self.document
    }

    #[cfg(feature = "viewer")]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn show(&mut self) -> anyhow::Result<&mut Self> {
        vsvg_viewer::show(self.document())?;
        Ok(self)
    }

    pub fn save(&self, path: impl AsRef<std::path::Path>) -> anyhow::Result<()> {
        let file = std::io::BufWriter::new(std::fs::File::create(path)?);
        self.document.to_svg(file)?;
        Ok(())
    }
}

impl Transforms for Sketch {
    fn transform(&mut self, affine: &Affine) -> &mut Self {
        if let Some(matrix) = self.transform_stack.last_mut() {
            *matrix *= *affine;
        } else {
            log::warn!("transform: no matrix on the stack");
        }

        self
    }
}

impl vsvg::Draw for Sketch {
    fn add_path<T: IntoBezPathTolerance>(&mut self, path: T) -> &mut Self {
        let mut path: Path =
            Path::from_tolerance_metadata(path, self.tolerance, self.path_metadata.clone());

        if let Some(&matrix) = self.transform_stack.last() {
            path.apply_transform(matrix);
        } else {
            log::warn!("add_path: no matrix on the stack");
        }

        self.document.push_path(self.target_layer, path);
        self
    }
}
