use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::types::DataType;
use crate::value::{LinearColor, PrimitiveValue, Value};

/// Pin slot address in a node card.
/// Keeps fixed slots isolated from dynamically expanding variadic slots to prevent index-shift corruption in saved files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SlotId {
    /// Fixed immutable slot index (e.g. 0 = Input A, 1 = Input B)
    Fixed(u16),
    /// Dynamically expanding variadic slot index belonging to an isolated group
    Variadic { group: u16, index: u16 },
}

impl SlotId {
    pub const fn fixed(slot: u16) -> Self {
        Self::Fixed(slot)
    }

    pub const fn variadic(group: u16, index: u16) -> Self {
        Self::Variadic { group, index }
    }
}

impl std::fmt::Display for SlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(id) => write!(f, "Fixed({})", id),
            Self::Variadic { group, index } => write!(f, "Variadic(g:{}, i:{})", group, index),
        }
    }
}

/// Visual shape of a pin socket on the node boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PinShape {
    /// Circular socket for continuous numeric & typed data (Float, Int, Vec3, Color)
    #[default]
    Circle,
    /// Arrow / Chevron socket for execution flow / pulse triggers
    Chevron,
    /// Diamond socket for discrete events / signals
    Diamond,
    /// Square socket for lists, buffers, arrays
    Square,
    /// Capsule / Pill socket for heavy references and asset handles
    Capsule,
}

/// Declarative schema of an input or output pin on a node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PinDescriptor {
    pub slot: SlotId,
    pub name: Cow<'static, str>,
    pub data_type: DataType,
    pub default_value: PrimitiveValue,
    pub is_variadic: bool,
    pub shape: PinShape,
}

impl PinDescriptor {
    pub fn new(slot: SlotId, name: impl Into<Cow<'static, str>>, data_type: DataType) -> Self {
        Self {
            slot,
            name: name.into(),
            data_type,
            default_value: PrimitiveValue::None,
            is_variadic: false,
            shape: PinShape::Circle,
        }
    }

    pub fn with_default(mut self, default_val: PrimitiveValue) -> Self {
        self.default_value = default_val;
        self
    }

    pub fn as_variadic(mut self) -> Self {
        self.is_variadic = true;
        self
    }

    pub fn with_shape(mut self, shape: PinShape) -> Self {
        self.shape = shape;
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Type mismatch on slot {slot}: expected {expected:?}, found {found:?}")]
    TypeMismatch {
        slot: SlotId,
        expected: DataType,
        found: DataType,
    },

    #[error("Missing input on slot {0}")]
    InputMissing(SlotId),

    #[error("Execution was cancelled by user")]
    Cancelled,

    #[error("Evaluation error: {0}")]
    Custom(String),
}

/// Context passed to asynchronous background tasks for cancellation and live progress monitoring.
#[derive(Debug, Default)]
pub struct TaskContext {
    pub cancel_token: Arc<AtomicBool>,
    pub progress: Arc<AtomicU32>, // 0..=1000 for 0.0% to 100.0%
}

impl TaskContext {
    pub fn new() -> Self {
        Self {
            cancel_token: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.load(Ordering::Relaxed)
    }

    pub fn cancel(&self) {
        self.cancel_token.store(true, Ordering::Relaxed);
    }

    pub fn set_progress(&self, fraction: f32) {
        let val = (fraction.clamp(0.0, 1.0) * 1000.0) as u32;
        self.progress.store(val, Ordering::Relaxed);
    }

    pub fn get_progress(&self) -> f32 {
        self.progress.load(Ordering::Relaxed) as f32 / 1000.0
    }
}

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The compute logic contract of a node: fast in-thread synchronous kernel,
/// or thread-pool asynchronous kernel with owned inputs.
pub enum NodeKernel {
    /// O(1) in-thread synchronous execution for lightweight math, logic, and pure transforms.
    Sync(Arc<dyn Fn(&[Value], &mut [Value]) -> Result<(), ExecutionError> + Send + Sync>),

    /// Background task execution for heavy CPU/GPU operations (mesh deformation, file loading, baking).
    Async(
        Arc<
            dyn Fn(
                    Vec<Value>,
                    Arc<TaskContext>,
                ) -> BoxFuture<'static, Result<Vec<Value>, ExecutionError>>
                + Send
                + Sync,
        >,
    ),
}

/// Category metadata for organizing nodes in search palettes, menus, and graphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeCategory {
    /// Hierarchical category path (e.g. "Math/Arithmetic", "Math/Trigonometry", "Logic", "Vector")
    pub path: Cow<'static, str>,
    /// Display tag for UI badges (e.g. "MATH", "TRIG", "VEC3", "LOGIC")
    pub tag: Cow<'static, str>,
    /// Optional accent color override for this category
    pub color: Option<LinearColor>,
}

impl NodeCategory {
    pub fn new(path: impl Into<Cow<'static, str>>) -> Self {
        let path = path.into();
        let tag = Self::default_tag_for_path(&path);
        Self {
            path,
            tag: Cow::Borrowed(tag),
            color: None,
        }
    }

    pub fn with_tag(mut self, tag: impl Into<Cow<'static, str>>) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn with_color(mut self, color: LinearColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn primary_group(&self) -> &str {
        self.path.split('/').next().unwrap_or(self.path.as_ref())
    }

    pub fn sub_group(&self) -> Option<&str> {
        let mut parts = self.path.split('/');
        parts.next();
        parts.next()
    }

    fn default_tag_for_path(path: &str) -> &'static str {
        if path.contains("Arithmetic") {
            "MATH"
        } else if path.contains("Trigonometry") {
            "TRIG"
        } else if path.contains("Vector") {
            "VEC3"
        } else if path.contains("Rounding") {
            "ROUND"
        } else if path.contains("Interpolation") {
            "LERP"
        } else if path.contains("Exponential") {
            "EXP"
        } else if path.contains("Comparison") {
            "CMP"
        } else {
            "LOGIC"
        }
    }
}

impl Default for NodeCategory {
    fn default() -> Self {
        Self::new("General")
    }
}

/// Defines what the inner body of the node card contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NodeBodyStyle {
    /// Pure empty body: only header and input/output pin sockets are drawn.
    /// No hardcoded preview, no dummy graphs, completely clean frame.
    #[default]
    Empty,

    /// Displays inline editable scalar parameter fields for disconnected inputs.
    Parameters,

    /// Reserved slot for custom user widget injection.
    CustomSlot,
}

/// Visual presentation and UI styling configuration passed by the node author.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NodeUiMetadata {
    /// Accent color for the header strip, socket glows, and badge
    pub accent_color: Option<LinearColor>,
    /// Background color for the header banner
    pub header_bg: Option<LinearColor>,
    /// Minimum card dimensions [width, height] in logical pixels
    pub min_size: Option<[f32; 2]>,
    /// What the body of the node card renders (default is Empty)
    pub body_style: NodeBodyStyle,
    /// Category classification & styling
    pub category: NodeCategory,
}

/// Declarative blueprint of a node registered in the engine.
#[derive(Clone)]
pub struct NodeDescriptor {
    pub type_id: Cow<'static, str>,
    pub version: u32,
    pub title: Cow<'static, str>,
    pub category: Cow<'static, str>,
    pub is_pure: bool,
    pub inputs: Vec<PinDescriptor>,
    pub outputs: Vec<PinDescriptor>,
    pub kernel: Arc<NodeKernel>,
    pub ui: NodeUiMetadata,
}

impl NodeDescriptor {
    pub fn new_sync(
        type_id: impl Into<Cow<'static, str>>,
        title: impl Into<Cow<'static, str>>,
        category: impl Into<Cow<'static, str>>,
        kernel: impl Fn(&[Value], &mut [Value]) -> Result<(), ExecutionError> + Send + Sync + 'static,
    ) -> Self {
        let cat_cow: Cow<'static, str> = category.into();
        let cat_struct = NodeCategory::new(cat_cow.clone());
        Self {
            type_id: type_id.into(),
            version: 1,
            title: title.into(),
            category: cat_cow,
            is_pure: true,
            inputs: Vec::new(),
            outputs: Vec::new(),
            kernel: Arc::new(NodeKernel::Sync(Arc::new(kernel))),
            ui: NodeUiMetadata {
                accent_color: None,
                header_bg: None,
                min_size: None,
                body_style: NodeBodyStyle::Empty,
                category: cat_struct,
            },
        }
    }

    pub fn new_async(
        type_id: impl Into<Cow<'static, str>>,
        title: impl Into<Cow<'static, str>>,
        category: impl Into<Cow<'static, str>>,
        kernel: impl Fn(
                Vec<Value>,
                Arc<TaskContext>,
            ) -> BoxFuture<'static, Result<Vec<Value>, ExecutionError>>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        let cat_cow: Cow<'static, str> = category.into();
        let cat_struct = NodeCategory::new(cat_cow.clone());
        Self {
            type_id: type_id.into(),
            version: 1,
            title: title.into(),
            category: cat_cow,
            is_pure: false,
            inputs: Vec::new(),
            outputs: Vec::new(),
            kernel: Arc::new(NodeKernel::Async(Arc::new(kernel))),
            ui: NodeUiMetadata {
                accent_color: None,
                header_bg: None,
                min_size: None,
                body_style: NodeBodyStyle::Empty,
                category: cat_struct,
            },
        }
    }

    pub fn with_version(mut self, ver: u32) -> Self {
        self.version = ver;
        self
    }

    pub fn with_purity(mut self, is_pure: bool) -> Self {
        self.is_pure = is_pure;
        self
    }

    pub fn with_accent_color(mut self, color: LinearColor) -> Self {
        self.ui.accent_color = Some(color);
        self
    }

    pub fn with_header_bg(mut self, color: LinearColor) -> Self {
        self.ui.header_bg = Some(color);
        self
    }

    pub fn with_category(mut self, category: NodeCategory) -> Self {
        self.category = category.path.clone();
        self.ui.category = category;
        self
    }

    pub fn with_body_style(mut self, style: NodeBodyStyle) -> Self {
        self.ui.body_style = style;
        self
    }

    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.ui.min_size = Some([width, height]);
        self
    }

    pub fn with_input(
        mut self,
        name: impl Into<Cow<'static, str>>,
        data_type: DataType,
        default_val: PrimitiveValue,
    ) -> Self {
        let slot = SlotId::Fixed(self.inputs.len() as u16);
        self.inputs.push(PinDescriptor {
            slot,
            name: name.into(),
            data_type,
            default_value: default_val,
            is_variadic: false,
            shape: PinShape::Circle,
        });
        self
    }

    pub fn with_input_full(
        mut self,
        name: impl Into<Cow<'static, str>>,
        data_type: DataType,
        default_val: PrimitiveValue,
        shape: PinShape,
    ) -> Self {
        let slot = SlotId::Fixed(self.inputs.len() as u16);
        self.inputs.push(PinDescriptor {
            slot,
            name: name.into(),
            data_type,
            default_value: default_val,
            is_variadic: false,
            shape,
        });
        self
    }

    pub fn with_output(
        mut self,
        name: impl Into<Cow<'static, str>>,
        data_type: DataType,
    ) -> Self {
        let slot = SlotId::Fixed(self.outputs.len() as u16);
        self.outputs.push(PinDescriptor {
            slot,
            name: name.into(),
            data_type,
            default_value: PrimitiveValue::None,
            is_variadic: false,
            shape: PinShape::Circle,
        });
        self
    }

    pub fn with_output_full(
        mut self,
        name: impl Into<Cow<'static, str>>,
        data_type: DataType,
        shape: PinShape,
    ) -> Self {
        let slot = SlotId::Fixed(self.outputs.len() as u16);
        self.outputs.push(PinDescriptor {
            slot,
            name: name.into(),
            data_type,
            default_value: PrimitiveValue::None,
            is_variadic: false,
            shape,
        });
        self
    }

    /// Executes the node kernel if it is synchronous.
    pub fn execute_sync(&self, inputs: &[Value], outputs: &mut [Value]) -> Result<(), ExecutionError> {
        match &*self.kernel {
            NodeKernel::Sync(f) => f(inputs, outputs),
            NodeKernel::Async(_) => Err(ExecutionError::Custom("Cannot synchronously execute async node".into())),
        }
    }
}

