use std::borrow::Cow;
use serde::{Deserialize, Serialize};

use crate::value::{LinearColor, PrimitiveValue, Value};

/// Strongly typed socket descriptors and polymorphic type variables.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataType {
    /// Polymorphic type variable (e.g. TypeVar(0) for 'T').
    /// Used by generic nodes like Reroute, Switch, List Append.
    TypeVar(u16),

    // Primitives
    Bool,
    Int,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Color,
    String,

    // Domain Specific
    AssetRef,
    Mesh,
    Material,
    Entity,

    // Containers & Extensibility
    List(Box<DataType>),
    Custom(Cow<'static, str>),
}

impl DataType {
    pub const ANY: Self = Self::TypeVar(0);

    pub fn custom(name: &'static str) -> Self {
        Self::Custom(Cow::Borrowed(name))
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeVar(i) => write!(f, "T{}", i),
            Self::Bool => write!(f, "Bool"),
            Self::Int => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::Vec2 => write!(f, "Vec2"),
            Self::Vec3 => write!(f, "Vec3"),
            Self::Vec4 => write!(f, "Vec4"),
            Self::Color => write!(f, "Color"),
            Self::String => write!(f, "String"),
            Self::AssetRef => write!(f, "AssetRef"),
            Self::Mesh => write!(f, "Mesh"),
            Self::Material => write!(f, "Material"),
            Self::Entity => write!(f, "Entity"),
            Self::List(inner) => write!(f, "List<{}>", inner),
            Self::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl DataType {

    /// Checks if a source wire type can connect to a target socket.
    pub fn can_connect_to(&self, target: &DataType) -> bool {
        if *self == *target {
            return true;
        }

        // Polymorphic Type Variables can connect to anything during initial drafting
        if matches!(self, DataType::TypeVar(_)) || matches!(target, DataType::TypeVar(_)) {
            return true;
        }

        match (self, target) {
            // Numeric and vector promotions
            (DataType::Int, DataType::Float) => true,
            (DataType::Float, DataType::Vec2) => true,
            (DataType::Float, DataType::Vec3) => true,
            (DataType::Float, DataType::Vec4) => true,

            // Color / Vector interop
            (DataType::Color, DataType::Vec4) => true,
            (DataType::Vec4, DataType::Color) => true,

            // Scalar -> List Promotion (e.g. Float -> List<Float>)
            (scalar, DataType::List(inner)) if scalar.can_connect_to(inner) => true,

            // Note: List -> Scalar is STRICTLY REJECTED to prevent silent data loss bugs!
            _ => false,
        }
    }

    /// Returns a human-friendly name for UI badges and tooltips.
    pub fn display_name(&self) -> &str {
        match self {
            DataType::TypeVar(_) => "Any",
            DataType::Bool => "Bool",
            DataType::Int => "Int",
            DataType::Float => "Float",
            DataType::Vec2 => "Vec2",
            DataType::Vec3 => "Vec3",
            DataType::Vec4 => "Vec4",
            DataType::Color => "Color",
            DataType::String => "String",
            DataType::AssetRef => "AssetRef",
            DataType::Mesh => "Mesh",
            DataType::Material => "Material",
            DataType::Entity => "Entity",
            DataType::List(_) => "List",
            DataType::Custom(name) => name.as_ref(),
        }
    }
}

/// Strict conversion function according to the single source of truth coercion table.
pub fn coerce(value: &Value, target: &DataType) -> Option<Value> {
    match (value, target) {
        // Direct matching
        (Value::Primitive(p), expected) if is_primitive_matching(p, expected) => {
            Some(Value::Primitive(p.clone()))
        }
        (Value::AssetRef(path), DataType::AssetRef) => Some(Value::AssetRef(path.clone())),
        (Value::Resource { type_id, .. }, expected) if type_id == expected => Some(value.clone()),

        // TypeVar acts as passthrough
        (_, DataType::TypeVar(_)) => Some(value.clone()),

        // Int -> Float
        (Value::Primitive(PrimitiveValue::Int(i)), DataType::Float) => {
            Some(Value::float(*i as f32))
        }

        // Float -> Vector Broadcasts
        (Value::Primitive(PrimitiveValue::Float(f)), DataType::Vec2) => {
            Some(Value::vec2(glam::Vec2::splat(*f)))
        }
        (Value::Primitive(PrimitiveValue::Float(f)), DataType::Vec3) => {
            Some(Value::vec3(glam::Vec3::splat(*f)))
        }
        (Value::Primitive(PrimitiveValue::Float(f)), DataType::Vec4) => {
            Some(Value::vec4(glam::Vec4::splat(*f)))
        }

        // Color <-> Vec4
        (Value::Primitive(PrimitiveValue::Color(c)), DataType::Vec4) => {
            Some(Value::vec4(glam::Vec4::new(c.r, c.g, c.b, c.a)))
        }
        (Value::Primitive(PrimitiveValue::Vec4(v)), DataType::Color) => {
            Some(Value::color(LinearColor::new(v.x, v.y, v.z, v.w)))
        }

        // Scalar -> List Promotion: wrap in a single-element list resource
        (val, DataType::List(inner)) if val.type_matches(inner) => {
            let list = vec![val.clone()];
            Some(Value::new_resource(
                DataType::List(inner.clone()),
                val.footprint_bytes() + std::mem::size_of::<Vec<Value>>(),
                list,
            ))
        }

        // Any other conversion (including List -> Scalar) is rejected
        _ => None,
    }
}

impl Value {
    /// Returns the corresponding DataType of this value.
    pub fn data_type(&self) -> DataType {
        match self {
            Value::Primitive(p) => match p {
                PrimitiveValue::None => DataType::TypeVar(0),
                PrimitiveValue::Bool(_) => DataType::Bool,
                PrimitiveValue::Int(_) => DataType::Int,
                PrimitiveValue::Float(_) => DataType::Float,
                PrimitiveValue::Vec2(_) => DataType::Vec2,
                PrimitiveValue::Vec3(_) => DataType::Vec3,
                PrimitiveValue::Vec4(_) => DataType::Vec4,
                PrimitiveValue::Color(_) => DataType::Color,
                PrimitiveValue::String(_) => DataType::String,
            },
            Value::AssetRef(_) => DataType::AssetRef,
            Value::Resource { type_id, .. } => type_id.clone(),
        }
    }

    /// Checks if this value matches a target DataType.
    pub fn type_matches(&self, target: &DataType) -> bool {
        self.data_type() == *target || matches!(target, DataType::TypeVar(_))
    }
}

fn is_primitive_matching(p: &PrimitiveValue, t: &DataType) -> bool {
    matches!(
        (p, t),
        (PrimitiveValue::Bool(_), DataType::Bool)
            | (PrimitiveValue::Int(_), DataType::Int)
            | (PrimitiveValue::Float(_), DataType::Float)
            | (PrimitiveValue::Vec2(_), DataType::Vec2)
            | (PrimitiveValue::Vec3(_), DataType::Vec3)
            | (PrimitiveValue::Vec4(_), DataType::Vec4)
            | (PrimitiveValue::Color(_), DataType::Color)
            | (PrimitiveValue::String(_), DataType::String)
            | (PrimitiveValue::None, DataType::TypeVar(_))
    )
}
