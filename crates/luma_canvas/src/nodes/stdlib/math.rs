use luma_dataflow::descriptor::{NodeBodyStyle, NodeCategory, NodeDescriptor};
use luma_dataflow::registry::NodeRegistry;
use luma_dataflow::types::DataType;
use luma_dataflow::value::{LinearColor, PrimitiveValue, Value};

#[inline(always)]
fn get_float(inputs: &[Value], idx: usize, default: f32) -> f32 {
    match inputs.get(idx) {
        Some(Value::Primitive(PrimitiveValue::Float(f))) => *f,
        Some(Value::Primitive(PrimitiveValue::Int(i))) => *i as f32,
        _ => default,
    }
}

#[inline(always)]
fn get_vec3(inputs: &[Value], idx: usize, default: glam::Vec3) -> glam::Vec3 {
    match inputs.get(idx) {
        Some(Value::Primitive(PrimitiveValue::Vec3(v))) => *v,
        _ => default,
    }
}

/// Registers the comprehensive suite of pure mathematical nodes into the engine catalog.
pub fn register_math_nodes(registry: &mut NodeRegistry) {
    // =========================================================================
    // 0. Constants & Inputs (القيم الثابتة والمداخل للبدء)
    // =========================================================================

    // math.number: Number / Float value source to start calculations
    registry.register(
        NodeDescriptor::new_sync("math.number", "Number", "Math/Constants", |inputs, outputs| {
            let val = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(val);
            Ok(())
        })
        .with_category(NodeCategory::new("Math/Constants").with_tag("NUM"))
        .with_body_style(NodeBodyStyle::Parameters)
        .with_accent_color(LinearColor::new(0.2, 0.7, 0.9, 1.0))
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("output", DataType::Float),
    );

    // math.integer: Integer value source
    registry.register(
        NodeDescriptor::new_sync("math.integer", "Integer", "Math/Constants", |inputs, outputs| {
            let val = match inputs.get(0) {
                Some(Value::Primitive(PrimitiveValue::Int(i))) => *i,
                Some(Value::Primitive(PrimitiveValue::Float(f))) => *f as i64,
                _ => 0,
            };
            outputs[0] = Value::int(val);
            Ok(())
        })
        .with_category(NodeCategory::new("Math/Constants").with_tag("INT"))
        .with_body_style(NodeBodyStyle::Parameters)
        .with_accent_color(LinearColor::new(0.25, 0.75, 0.85, 1.0))
        .with_input("value", DataType::Int, PrimitiveValue::Int(0))
        .with_output("output", DataType::Int),
    );


    // math.constant_pi: Mathematical constant Pi (3.14159...)
    registry.register(
        NodeDescriptor::new_sync("math.constant_pi", "Constant Pi (π)", "Math/Constants", |_inputs, outputs| {
            outputs[0] = Value::float(std::f32::consts::PI);
            Ok(())
        })
        .with_category(NodeCategory::new("Math/Constants").with_tag("CONST"))
        .with_output("pi", DataType::Float),
    );

    // math.constant_e: Mathematical constant e (2.71828...)
    registry.register(
        NodeDescriptor::new_sync("math.constant_e", "Constant E (e)", "Math/Constants", |_inputs, outputs| {
            outputs[0] = Value::float(std::f32::consts::E);
            Ok(())
        })
        .with_category(NodeCategory::new("Math/Constants").with_tag("CONST"))
        .with_output("e", DataType::Float),
    );

    // math.constant_tau: Mathematical constant Tau (2*pi)
    registry.register(
        NodeDescriptor::new_sync("math.constant_tau", "Constant Tau (τ)", "Math/Constants", |_inputs, outputs| {
            outputs[0] = Value::float(std::f32::consts::TAU);
            Ok(())
        })
        .with_category(NodeCategory::new("Math/Constants").with_tag("CONST"))
        .with_output("tau", DataType::Float),
    );
    // =========================================================================
    // 1. Basic Arithmetic Operations (العمليات الحسابية الأساسية)
    // =========================================================================

    // math.add: a + b
    registry.register(
        NodeDescriptor::new_sync("math.add", "Add", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(a + b);
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.subtract: a - b
    registry.register(
        NodeDescriptor::new_sync("math.subtract", "Subtract", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(a - b);
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.multiply: a * b
    registry.register(
        NodeDescriptor::new_sync("math.multiply", "Multiply", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 1.0);
            let b = get_float(inputs, 1, 1.0);
            outputs[0] = Value::float(a * b);
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(1.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // math.divide: a / b (safe division by zero returns 0.0)
    registry.register(
        NodeDescriptor::new_sync("math.divide", "Divide", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 1.0);
            let b = get_float(inputs, 1, 1.0);
            let res = if b.abs() < f32::EPSILON { 0.0 } else { a / b };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(1.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // math.modulo: a % b
    registry.register(
        NodeDescriptor::new_sync("math.modulo", "Modulo", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 1.0);
            let res = if b.abs() < f32::EPSILON { 0.0 } else { a % b };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // math.power: a^b
    registry.register(
        NodeDescriptor::new_sync("math.power", "Power", "Math/Arithmetic", |inputs, outputs| {
            let base = get_float(inputs, 0, 1.0);
            let exp = get_float(inputs, 1, 2.0);
            outputs[0] = Value::float(base.powf(exp));
            Ok(())
        })
        .with_input("base", DataType::Float, PrimitiveValue::Float(1.0))
        .with_input("exponent", DataType::Float, PrimitiveValue::Float(2.0))
        .with_output("result", DataType::Float),
    );

    // math.sqrt: sqrt(a)
    registry.register(
        NodeDescriptor::new_sync("math.sqrt", "Square Root", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0).max(0.0);
            outputs[0] = Value::float(a.sqrt());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.abs: |a|
    registry.register(
        NodeDescriptor::new_sync("math.abs", "Absolute", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.abs());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.negate: -a
    registry.register(
        NodeDescriptor::new_sync("math.negate", "Negate", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(-a);
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.sign: signum(a)
    registry.register(
        NodeDescriptor::new_sync("math.sign", "Sign", "Math/Arithmetic", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let res = if a > 0.0 { 1.0 } else if a < 0.0 { -1.0 } else { 0.0 };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // =========================================================================
    // 2. Rounding & Clamping (التقريب والقص)
    // =========================================================================

    // math.floor
    registry.register(
        NodeDescriptor::new_sync("math.floor", "Floor", "Math/Rounding", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.floor());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.ceil
    registry.register(
        NodeDescriptor::new_sync("math.ceil", "Ceil", "Math/Rounding", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.ceil());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.round
    registry.register(
        NodeDescriptor::new_sync("math.round", "Round", "Math/Rounding", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.round());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.fract
    registry.register(
        NodeDescriptor::new_sync("math.fract", "Fraction", "Math/Rounding", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.fract());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.min: min(a, b)
    registry.register(
        NodeDescriptor::new_sync("math.min", "Minimum", "Math/Rounding", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(a.min(b));
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.max: max(a, b)
    registry.register(
        NodeDescriptor::new_sync("math.max", "Maximum", "Math/Rounding", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(a.max(b));
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.clamp: clamp(val, min, max)
    registry.register(
        NodeDescriptor::new_sync("math.clamp", "Clamp", "Math/Rounding", |inputs, outputs| {
            let val = get_float(inputs, 0, 0.0);
            let min = get_float(inputs, 1, 0.0);
            let max = get_float(inputs, 2, 1.0);
            outputs[0] = Value::float(val.clamp(min, max));
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("min", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("max", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // =========================================================================
    // 3. Interpolation & Mapping (الاستيفاء والتدريج)
    // =========================================================================

    // math.lerp: a + (b - a) * t
    registry.register(
        NodeDescriptor::new_sync("math.lerp", "Lerp", "Math/Interpolation", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 1.0);
            let t = get_float(inputs, 2, 0.5);
            outputs[0] = Value::float(a + (b - a) * t);
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(1.0))
        .with_input("t", DataType::Float, PrimitiveValue::Float(0.5))
        .with_output("result", DataType::Float),
    );

    // math.smoothstep
    registry.register(
        NodeDescriptor::new_sync("math.smoothstep", "Smoothstep", "Math/Interpolation", |inputs, outputs| {
            let edge0 = get_float(inputs, 0, 0.0);
            let edge1 = get_float(inputs, 1, 1.0);
            let x = get_float(inputs, 2, 0.5);
            let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
            outputs[0] = Value::float(t * t * (3.0 - 2.0 * t));
            Ok(())
        })
        .with_input("edge0", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("edge1", DataType::Float, PrimitiveValue::Float(1.0))
        .with_input("x", DataType::Float, PrimitiveValue::Float(0.5))
        .with_output("result", DataType::Float),
    );

    // math.map_range: maps value from in_min..in_max to out_min..out_max
    registry.register(
        NodeDescriptor::new_sync("math.map_range", "Map Range", "Math/Interpolation", |inputs, outputs| {
            let val = get_float(inputs, 0, 0.5);
            let in_min = get_float(inputs, 1, 0.0);
            let in_max = get_float(inputs, 2, 1.0);
            let out_min = get_float(inputs, 3, 0.0);
            let out_max = get_float(inputs, 4, 100.0);

            let in_span = in_max - in_min;
            let res = if in_span.abs() < f32::EPSILON {
                out_min
            } else {
                let norm = (val - in_min) / in_span;
                out_min + norm * (out_max - out_min)
            };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.5))
        .with_input("in_min", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("in_max", DataType::Float, PrimitiveValue::Float(1.0))
        .with_input("out_min", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("out_max", DataType::Float, PrimitiveValue::Float(100.0))
        .with_output("result", DataType::Float),
    );

    // =========================================================================
    // 4. Trigonometry (علم المثلثات)
    // =========================================================================

    // math.sin
    registry.register(
        NodeDescriptor::new_sync("math.sin", "Sine", "Math/Trigonometry", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.sin());
            Ok(())
        })
        .with_input("angle_rad", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.cos
    registry.register(
        NodeDescriptor::new_sync("math.cos", "Cosine", "Math/Trigonometry", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.cos());
            Ok(())
        })
        .with_input("angle_rad", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.tan
    registry.register(
        NodeDescriptor::new_sync("math.tan", "Tangent", "Math/Trigonometry", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.tan());
            Ok(())
        })
        .with_input("angle_rad", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.asin
    registry.register(
        NodeDescriptor::new_sync("math.asin", "Arc Sine", "Math/Trigonometry", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0).clamp(-1.0, 1.0);
            outputs[0] = Value::float(a.asin());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.acos
    registry.register(
        NodeDescriptor::new_sync("math.acos", "Arc Cosine", "Math/Trigonometry", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0).clamp(-1.0, 1.0);
            outputs[0] = Value::float(a.acos());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.atan
    registry.register(
        NodeDescriptor::new_sync("math.atan", "Arc Tangent", "Math/Trigonometry", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.atan());
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.atan2: atan2(y, x)
    registry.register(
        NodeDescriptor::new_sync("math.atan2", "Arc Tangent 2", "Math/Trigonometry", |inputs, outputs| {
            let y = get_float(inputs, 0, 0.0);
            let x = get_float(inputs, 1, 1.0);
            outputs[0] = Value::float(y.atan2(x));
            Ok(())
        })
        .with_input("y", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("x", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // math.radians: deg -> rad
    registry.register(
        NodeDescriptor::new_sync("math.radians", "To Radians", "Math/Trigonometry", |inputs, outputs| {
            let deg = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(deg.to_radians());
            Ok(())
        })
        .with_input("degrees", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("radians", DataType::Float),
    );

    // math.degrees: rad -> deg
    registry.register(
        NodeDescriptor::new_sync("math.degrees", "To Degrees", "Math/Trigonometry", |inputs, outputs| {
            let rad = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(rad.to_degrees());
            Ok(())
        })
        .with_input("radians", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("degrees", DataType::Float),
    );

    // =========================================================================
    // 5. Exponentials & Logarithms (الأسس واللوغاريتمات)
    // =========================================================================

    // math.exp: e^a
    registry.register(
        NodeDescriptor::new_sync("math.exp", "Exponential", "Math/Exponential", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            outputs[0] = Value::float(a.exp());
            Ok(())
        })
        .with_input("exponent", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.log: ln(a)
    registry.register(
        NodeDescriptor::new_sync("math.log", "Natural Log", "Math/Exponential", |inputs, outputs| {
            let a = get_float(inputs, 0, 1.0);
            let res = if a <= 0.0 { 0.0 } else { a.ln() };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // math.log2: log2(a)
    registry.register(
        NodeDescriptor::new_sync("math.log2", "Log Base 2", "Math/Exponential", |inputs, outputs| {
            let a = get_float(inputs, 0, 1.0);
            let res = if a <= 0.0 { 0.0 } else { a.log2() };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // math.log10: log10(a)
    registry.register(
        NodeDescriptor::new_sync("math.log10", "Log Base 10", "Math/Exponential", |inputs, outputs| {
            let a = get_float(inputs, 0, 1.0);
            let res = if a <= 0.0 { 0.0 } else { a.log10() };
            outputs[0] = Value::float(res);
            Ok(())
        })
        .with_input("value", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Float),
    );

    // =========================================================================
    // 6. Comparison & Logic (المقارنة والمنطق)
    // =========================================================================

    // math.greater_than: a > b ? 1.0 : 0.0
    registry.register(
        NodeDescriptor::new_sync("math.greater_than", "Greater Than", "Math/Comparison", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(if a > b { 1.0 } else { 0.0 });
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.less_than: a < b ? 1.0 : 0.0
    registry.register(
        NodeDescriptor::new_sync("math.less_than", "Less Than", "Math/Comparison", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(if a < b { 1.0 } else { 0.0 });
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // math.equal: |a - b| < epsilon ? 1.0 : 0.0
    registry.register(
        NodeDescriptor::new_sync("math.equal", "Equal", "Math/Comparison", |inputs, outputs| {
            let a = get_float(inputs, 0, 0.0);
            let b = get_float(inputs, 1, 0.0);
            outputs[0] = Value::float(if (a - b).abs() < f32::EPSILON { 1.0 } else { 0.0 });
            Ok(())
        })
        .with_input("a", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("b", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("result", DataType::Float),
    );

    // =========================================================================
    // 7. Vector 3D Math Operations (عمليات المتجهات ثلاثية الأبعاد)
    // =========================================================================

    // math.vector_add: v1 + v2
    registry.register(
        NodeDescriptor::new_sync("math.vector_add", "Vector Add", "Math/Vector", |inputs, outputs| {
            let v1 = get_vec3(inputs, 0, glam::Vec3::ZERO);
            let v2 = get_vec3(inputs, 1, glam::Vec3::ZERO);
            outputs[0] = Value::vec3(v1 + v2);
            Ok(())
        })
        .with_input("a", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_input("b", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("result", DataType::Vec3),
    );

    // math.vector_subtract: v1 - v2
    registry.register(
        NodeDescriptor::new_sync("math.vector_subtract", "Vector Subtract", "Math/Vector", |inputs, outputs| {
            let v1 = get_vec3(inputs, 0, glam::Vec3::ZERO);
            let v2 = get_vec3(inputs, 1, glam::Vec3::ZERO);
            outputs[0] = Value::vec3(v1 - v2);
            Ok(())
        })
        .with_input("a", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_input("b", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("result", DataType::Vec3),
    );

    // math.vector_multiply: v1 * v2 (component-wise)
    registry.register(
        NodeDescriptor::new_sync("math.vector_multiply", "Vector Multiply", "Math/Vector", |inputs, outputs| {
            let v1 = get_vec3(inputs, 0, glam::Vec3::ONE);
            let v2 = get_vec3(inputs, 1, glam::Vec3::ONE);
            outputs[0] = Value::vec3(v1 * v2);
            Ok(())
        })
        .with_input("a", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ONE))
        .with_input("b", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ONE))
        .with_output("result", DataType::Vec3),
    );

    // math.vector_scale: v * s
    registry.register(
        NodeDescriptor::new_sync("math.vector_scale", "Vector Scale", "Math/Vector", |inputs, outputs| {
            let v = get_vec3(inputs, 0, glam::Vec3::ZERO);
            let s = get_float(inputs, 1, 1.0);
            outputs[0] = Value::vec3(v * s);
            Ok(())
        })
        .with_input("vector", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_input("scale", DataType::Float, PrimitiveValue::Float(1.0))
        .with_output("result", DataType::Vec3),
    );

    // math.vector_dot: v1 . v2
    registry.register(
        NodeDescriptor::new_sync("math.vector_dot", "Dot Product", "Math/Vector", |inputs, outputs| {
            let v1 = get_vec3(inputs, 0, glam::Vec3::ZERO);
            let v2 = get_vec3(inputs, 1, glam::Vec3::ZERO);
            outputs[0] = Value::float(v1.dot(v2));
            Ok(())
        })
        .with_input("a", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_input("b", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("dot", DataType::Float),
    );

    // math.vector_cross: v1 x v2
    registry.register(
        NodeDescriptor::new_sync("math.vector_cross", "Cross Product", "Math/Vector", |inputs, outputs| {
            let v1 = get_vec3(inputs, 0, glam::Vec3::ZERO);
            let v2 = get_vec3(inputs, 1, glam::Vec3::ZERO);
            outputs[0] = Value::vec3(v1.cross(v2));
            Ok(())
        })
        .with_input("a", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_input("b", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("cross", DataType::Vec3),
    );

    // math.vector_length: |v|
    registry.register(
        NodeDescriptor::new_sync("math.vector_length", "Vector Length", "Math/Vector", |inputs, outputs| {
            let v = get_vec3(inputs, 0, glam::Vec3::ZERO);
            outputs[0] = Value::float(v.length());
            Ok(())
        })
        .with_input("vector", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("length", DataType::Float),
    );

    // math.vector_distance: distance(v1, v2)
    registry.register(
        NodeDescriptor::new_sync("math.vector_distance", "Vector Distance", "Math/Vector", |inputs, outputs| {
            let v1 = get_vec3(inputs, 0, glam::Vec3::ZERO);
            let v2 = get_vec3(inputs, 1, glam::Vec3::ZERO);
            outputs[0] = Value::float(v1.distance(v2));
            Ok(())
        })
        .with_input("a", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_input("b", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("distance", DataType::Float),
    );

    // math.vector_normalize: v / |v|
    registry.register(
        NodeDescriptor::new_sync("math.vector_normalize", "Normalize", "Math/Vector", |inputs, outputs| {
            let v = get_vec3(inputs, 0, glam::Vec3::ZERO);
            outputs[0] = Value::vec3(v.normalize_or_zero());
            Ok(())
        })
        .with_input("vector", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("normalized", DataType::Vec3),
    );

    // math.vector_compose: (x, y, z -> Vec3)
    registry.register(
        NodeDescriptor::new_sync("math.vector_compose", "Compose Vec3", "Math/Vector", |inputs, outputs| {
            let x = get_float(inputs, 0, 0.0);
            let y = get_float(inputs, 1, 0.0);
            let z = get_float(inputs, 2, 0.0);
            outputs[0] = Value::vec3(glam::Vec3::new(x, y, z));
            Ok(())
        })
        .with_input("x", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("y", DataType::Float, PrimitiveValue::Float(0.0))
        .with_input("z", DataType::Float, PrimitiveValue::Float(0.0))
        .with_output("vector", DataType::Vec3),
    );

    // math.vector_decompose: (Vec3 -> x, y, z)
    registry.register(
        NodeDescriptor::new_sync("math.vector_decompose", "Decompose Vec3", "Math/Vector", |inputs, outputs| {
            let v = get_vec3(inputs, 0, glam::Vec3::ZERO);
            outputs[0] = Value::float(v.x);
            outputs[1] = Value::float(v.y);
            outputs[2] = Value::float(v.z);
            Ok(())
        })
        .with_input("vector", DataType::Vec3, PrimitiveValue::Vec3(glam::Vec3::ZERO))
        .with_output("x", DataType::Float)
        .with_output("y", DataType::Float)
        .with_output("z", DataType::Float),
    );
}
