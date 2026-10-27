use std::any::Any;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::types::DataType;

static GLOBAL_RESOURCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Monotonically increasing atomic identifier for resources in memory.
/// Guarantees that no two live or recycled resources share the same ID (eliminating the ABA trap).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceId(pub u64);

impl ResourceId {
    pub fn next() -> Self {
        Self(GLOBAL_RESOURCE_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Res({})", self.0)
    }
}

/// Pure mathematical linear color without dependency on external rendering engines.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinearColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for LinearColor {
    fn default() -> Self {
        Self::WHITE
    }
}

impl LinearColor {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn from_array(arr: [f32; 4]) -> Self {
        Self {
            r: arr[0],
            g: arr[1],
            b: arr[2],
            a: arr[3],
        }
    }
}

/// Lightweight primitive values that can be serialized and saved directly into .luma project files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValue {
    None,
    Bool(bool),
    Int(i64),
    Float(f32),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    Color(LinearColor),
    String(String),
}

impl PrimitiveValue {
    /// Total bitwise equality for floats and vectors with NaN canonicalization.
    /// Prevents NaN from causing infinite dirty-propagation loops in caching.
    pub fn total_eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PrimitiveValue::None, PrimitiveValue::None) => true,
            (PrimitiveValue::Bool(a), PrimitiveValue::Bool(b)) => a == b,
            (PrimitiveValue::Int(a), PrimitiveValue::Int(b)) => a == b,
            (PrimitiveValue::Float(a), PrimitiveValue::Float(b)) => float_bits_eq(*a, *b),
            (PrimitiveValue::Vec2(a), PrimitiveValue::Vec2(b)) => {
                float_bits_eq(a.x, b.x) && float_bits_eq(a.y, b.y)
            }
            (PrimitiveValue::Vec3(a), PrimitiveValue::Vec3(b)) => {
                float_bits_eq(a.x, b.x) && float_bits_eq(a.y, b.y) && float_bits_eq(a.z, b.z)
            }
            (PrimitiveValue::Vec4(a), PrimitiveValue::Vec4(b)) => {
                float_bits_eq(a.x, b.x)
                    && float_bits_eq(a.y, b.y)
                    && float_bits_eq(a.z, b.z)
                    && float_bits_eq(a.w, b.w)
            }
            (PrimitiveValue::Color(a), PrimitiveValue::Color(b)) => {
                float_bits_eq(a.r, b.r)
                    && float_bits_eq(a.g, b.g)
                    && float_bits_eq(a.b, b.b)
                    && float_bits_eq(a.a, b.a)
            }
            (PrimitiveValue::String(a), PrimitiveValue::String(b)) => a == b,
            _ => false,
        }
    }

    /// Computes a stable 64-bit hash fingerprint for cache change detection.
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        match self {
            PrimitiveValue::None => 0.hash(&mut hasher),
            PrimitiveValue::Bool(b) => {
                1.hash(&mut hasher);
                b.hash(&mut hasher);
            }
            PrimitiveValue::Int(i) => {
                2.hash(&mut hasher);
                i.hash(&mut hasher);
            }
            PrimitiveValue::Float(f) => {
                3.hash(&mut hasher);
                canonical_float_bits(*f).hash(&mut hasher);
            }
            PrimitiveValue::Vec2(v) => {
                4.hash(&mut hasher);
                canonical_float_bits(v.x).hash(&mut hasher);
                canonical_float_bits(v.y).hash(&mut hasher);
            }
            PrimitiveValue::Vec3(v) => {
                5.hash(&mut hasher);
                canonical_float_bits(v.x).hash(&mut hasher);
                canonical_float_bits(v.y).hash(&mut hasher);
                canonical_float_bits(v.z).hash(&mut hasher);
            }
            PrimitiveValue::Vec4(v) => {
                6.hash(&mut hasher);
                canonical_float_bits(v.x).hash(&mut hasher);
                canonical_float_bits(v.y).hash(&mut hasher);
                canonical_float_bits(v.z).hash(&mut hasher);
                canonical_float_bits(v.w).hash(&mut hasher);
            }
            PrimitiveValue::Color(c) => {
                7.hash(&mut hasher);
                canonical_float_bits(c.r).hash(&mut hasher);
                canonical_float_bits(c.g).hash(&mut hasher);
                canonical_float_bits(c.b).hash(&mut hasher);
                canonical_float_bits(c.a).hash(&mut hasher);
            }
            PrimitiveValue::String(s) => {
                8.hash(&mut hasher);
                s.hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Formats the primitive value into a concise, readable string for node card display.
    pub fn to_concise_string(&self) -> String {
        match self {
            Self::Float(f) => {
                if f.fract() == 0.0 {
                    format!("{:.1}", f)
                } else {
                    format!("{:.2}", f)
                }
            }
            Self::Int(i) => format!("{}", i),
            Self::Bool(b) => format!("{}", b),
            Self::Vec2(v) => format!("({:.1}, {:.1})", v.x, v.y),
            Self::Vec3(v) => format!("({:.1}, {:.1}, {:.1})", v.x, v.y, v.z),
            Self::Vec4(v) => format!("({:.1}, {:.1}, {:.1}, {:.1})", v.x, v.y, v.z, v.w),
            Self::Color(c) => format!("rgba({:.1}, {:.1}, {:.1})", c.r, c.g, c.b),
            Self::String(s) => {
                if s.len() > 8 {
                    format!("\"{}...\"", &s[..7])
                } else {
                    format!("\"{}\"", s)
                }
            }
            Self::None => String::new(),
        }
    }
}


#[inline(always)]
fn float_bits_eq(a: f32, b: f32) -> bool {
    canonical_float_bits(a) == canonical_float_bits(b)
}

#[inline(always)]
fn canonical_float_bits(f: f32) -> u32 {
    if f.is_nan() {
        0x7fc00000 // Canonical quiet NaN
    } else if f == 0.0 {
        0 // Canonicalize +0.0 and -0.0
    } else {
        f.to_bits()
    }
}

/// The universal runtime carrier for dataflow wire passing.
#[derive(Clone)]
pub enum Value {
    /// 1. Lightweight primitive that can be directly serialized
    Primitive(PrimitiveValue),

    /// 2. Lazy asset path passed through to Bevy's AssetServer without in-engine CPU modification
    AssetRef(String),

    /// 3. Evaluated live in-memory heavy resource (Mesh geometry, texture pixels, physics shape)
    Resource {
        id: ResourceId,
        footprint_bytes: usize,
        type_id: DataType,
        data: Arc<dyn Any + Send + Sync>,
    },
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Primitive(p) => write!(f, "Value::Primitive({:?})", p),
            Self::AssetRef(s) => write!(f, "Value::AssetRef({:?})", s),
            Self::Resource { id, footprint_bytes, type_id, .. } => {
                f.debug_struct("Value::Resource")
                    .field("id", id)
                    .field("footprint_bytes", footprint_bytes)
                    .field("type_id", type_id)
                    .finish_non_exhaustive()
            }
        }
    }
}

impl Value {

    /// Factory for creating an evaluated heavy resource wrapped in an Arc.
    pub fn new_resource<T: Send + Sync + 'static>(
        type_id: DataType,
        footprint_bytes: usize,
        data: T,
    ) -> Self {
        Self::Resource {
            id: ResourceId::next(),
            footprint_bytes,
            type_id,
            data: Arc::new(data),
        }
    }

    /// Convenience helpers
    pub fn none() -> Self {
        Self::Primitive(PrimitiveValue::None)
    }

    pub fn bool(b: bool) -> Self {
        Self::Primitive(PrimitiveValue::Bool(b))
    }

    pub fn int(i: i64) -> Self {
        Self::Primitive(PrimitiveValue::Int(i))
    }

    pub fn float(f: f32) -> Self {
        Self::Primitive(PrimitiveValue::Float(f))
    }

    pub fn vec2(v: glam::Vec2) -> Self {
        Self::Primitive(PrimitiveValue::Vec2(v))
    }

    pub fn vec3(v: glam::Vec3) -> Self {
        Self::Primitive(PrimitiveValue::Vec3(v))
    }

    pub fn vec4(v: glam::Vec4) -> Self {
        Self::Primitive(PrimitiveValue::Vec4(v))
    }

    pub fn color(c: LinearColor) -> Self {
        Self::Primitive(PrimitiveValue::Color(c))
    }

    pub fn string(s: impl Into<String>) -> Self {
        Self::Primitive(PrimitiveValue::String(s.into()))
    }

    pub fn asset_ref(path: impl Into<String>) -> Self {
        Self::AssetRef(path.into())
    }

    /// Downcasts a resource to its concrete type if matching.
    pub fn downcast_resource<T: 'static>(&self) -> Option<&T> {
        match self {
            Self::Resource { data, .. } => data.downcast_ref::<T>(),
            _ => None,
        }
    }

    /// Returns the exact memory footprint in bytes for memory-budgeted LRU cache eviction.
    pub fn footprint_bytes(&self) -> usize {
        match self {
            Self::Primitive(p) => match p {
                PrimitiveValue::String(s) => std::mem::size_of::<PrimitiveValue>() + s.len(),
                _ => std::mem::size_of::<PrimitiveValue>(),
            },
            Self::AssetRef(s) => std::mem::size_of::<String>() + s.len(),
            Self::Resource { footprint_bytes, .. } => *footprint_bytes,
        }
    }

    /// Returns true if this is a heavy in-memory resource eligible for LRU cache eviction under memory pressure.
    pub fn is_heavy_resource(&self) -> bool {
        matches!(self, Self::Resource { .. })
    }

    /// Computes a stable 64-bit fingerprint for change detection and early-exit memoization.
    pub fn fingerprint(&self) -> u64 {
        match self {
            Self::Primitive(p) => p.fingerprint(),
            Self::AssetRef(path) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                "AssetRef".hash(&mut hasher);
                path.hash(&mut hasher);
                hasher.finish()
            }
            Self::Resource { id, .. } => id.0, // Monotonic never-reused ID!
        }
    }

    /// Checks if two values are equal for early-exit memoization.
    pub fn matches_cached(&self, old: &Self) -> bool {
        match (self, old) {
            (Self::Primitive(a), Self::Primitive(b)) => a.total_eq(b),
            (Self::AssetRef(a), Self::AssetRef(b)) => a == b,
            (Self::Resource { id: id_a, data: data_a, .. }, Self::Resource { id: id_b, data: data_b, .. }) => {
                id_a == id_b || Arc::ptr_eq(data_a, data_b)
            }
            _ => false,
        }
    }

    /// Formats the value concisely for in-node UI display.
    pub fn to_concise_string(&self) -> String {
        match self {
            Self::Primitive(p) => p.to_concise_string(),
            Self::AssetRef(s) => format!("Asset({})", s),
            Self::Resource { type_id, .. } => format!("[{:?}]", type_id),
        }
    }
}

/// Trait for converting a Rust value into an engine `Value` with type metadata.
pub trait IntoNodeValue {
    fn into_node_value(self) -> Value;
    fn node_data_type() -> DataType;
}

/// Trait for extracting a Rust value from an engine `Value`.
pub trait FromNodeValue: Sized {
    fn from_node_value(val: &Value) -> Option<Self>;
    fn node_data_type() -> DataType;
}

/// Marker trait allowing any user-defined struct to automatically become an engine resource.
pub trait CustomNodeResource: Send + Sync + Clone + 'static {
    fn type_name() -> &'static str;
}

impl<T: CustomNodeResource> IntoNodeValue for T {
    fn into_node_value(self) -> Value {
        Value::new_resource(
            DataType::custom(T::type_name()),
            std::mem::size_of::<T>(),
            self,
        )
    }
    fn node_data_type() -> DataType {
        DataType::custom(T::type_name())
    }
}

impl<T: CustomNodeResource> FromNodeValue for T {
    fn from_node_value(val: &Value) -> Option<Self> {
        val.downcast_resource::<T>().cloned()
    }
    fn node_data_type() -> DataType {
        DataType::custom(T::type_name())
    }
}

impl IntoNodeValue for f32 {
    fn into_node_value(self) -> Value {
        Value::float(self)
    }
    fn node_data_type() -> DataType {
        DataType::Float
    }
}

impl FromNodeValue for f32 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Float(f)) => Some(*f),
            Value::Primitive(PrimitiveValue::Int(i)) => Some(*i as f32),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Float
    }
}

impl IntoNodeValue for f64 {
    fn into_node_value(self) -> Value {
        Value::float(self as f32)
    }
    fn node_data_type() -> DataType {
        DataType::Float
    }
}

impl FromNodeValue for f64 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Float(f)) => Some(*f as f64),
            Value::Primitive(PrimitiveValue::Int(i)) => Some(*i as f64),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Float
    }
}

impl IntoNodeValue for i64 {
    fn into_node_value(self) -> Value {
        Value::int(self)
    }
    fn node_data_type() -> DataType {
        DataType::Int
    }
}

impl FromNodeValue for i64 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Int(i)) => Some(*i),
            Value::Primitive(PrimitiveValue::Float(f)) => Some(*f as i64),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Int
    }
}

impl IntoNodeValue for i32 {
    fn into_node_value(self) -> Value {
        Value::int(self as i64)
    }
    fn node_data_type() -> DataType {
        DataType::Int
    }
}

impl FromNodeValue for i32 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Int(i)) => Some(*i as i32),
            Value::Primitive(PrimitiveValue::Float(f)) => Some(*f as i32),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Int
    }
}

impl IntoNodeValue for bool {
    fn into_node_value(self) -> Value {
        Value::bool(self)
    }
    fn node_data_type() -> DataType {
        DataType::Bool
    }
}

impl FromNodeValue for bool {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Bool(b)) => Some(*b),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Bool
    }
}

impl IntoNodeValue for String {
    fn into_node_value(self) -> Value {
        Value::string(self)
    }
    fn node_data_type() -> DataType {
        DataType::String
    }
}

impl FromNodeValue for String {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::String(s)) => Some(s.clone()),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::String
    }
}

impl IntoNodeValue for LinearColor {
    fn into_node_value(self) -> Value {
        Value::color(self)
    }
    fn node_data_type() -> DataType {
        DataType::Color
    }
}

impl FromNodeValue for LinearColor {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Color(c)) => Some(*c),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Color
    }
}

impl IntoNodeValue for glam::Vec2 {
    fn into_node_value(self) -> Value {
        Value::vec2(self)
    }
    fn node_data_type() -> DataType {
        DataType::Vec2
    }
}

impl FromNodeValue for glam::Vec2 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Vec2(v)) => Some(*v),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Vec2
    }
}

impl IntoNodeValue for glam::Vec3 {
    fn into_node_value(self) -> Value {
        Value::vec3(self)
    }
    fn node_data_type() -> DataType {
        DataType::Vec3
    }
}

impl FromNodeValue for glam::Vec3 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Vec3(v)) => Some(*v),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Vec3
    }
}

impl IntoNodeValue for glam::Vec4 {
    fn into_node_value(self) -> Value {
        Value::vec4(self)
    }
    fn node_data_type() -> DataType {
        DataType::Vec4
    }
}

impl FromNodeValue for glam::Vec4 {
    fn from_node_value(val: &Value) -> Option<Self> {
        match val {
            Value::Primitive(PrimitiveValue::Vec4(v)) => Some(*v),
            _ => None,
        }
    }
    fn node_data_type() -> DataType {
        DataType::Vec4
    }
}

